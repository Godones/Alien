use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use unifs::inode::{basic_file_stat, UniFsDirInode};
use vfscore::{
    error::VfsError,
    file::VfsFile,
    impl_dir_inode_default,
    inode::{InodeAttr, VfsInode},
    superblock::VfsSuperBlock,
    utils::{
        VfsDirEntry, VfsInodeMode, VfsNodePerm, VfsNodeType, VfsRenameFlag, VfsTime, VfsTimeSpec,
    },
    RRefVec, VfsResult,
};

use super::*;
use crate::inode::{file::RamFsFileInode, symlink::RamFsSymLinkInode};
pub struct RamFsDirInode<T: Send + Sync, R: VfsRawMutex> {
    inode: UniFsDirInode<T, R>,
    ext_attr: lock_api::Mutex<R, BTreeMap<String, String>>,
}

impl<T: RamFsProvider + 'static, R: VfsRawMutex + 'static> RamFsDirInode<T, R> {
    pub fn new(
        sb: &Arc<UniFsSuperBlock<R>>,
        provider: T,
        inode_number: u64,
        perm: VfsNodePerm,
    ) -> Self {
        Self {
            inode: UniFsDirInode {
                basic: UniFsInodeSame::new(sb, provider, inode_number, perm),
                children: lock_api::Mutex::new(Vec::new()),
            },
            ext_attr: lock_api::Mutex::new(BTreeMap::new()),
        }
    }
    pub fn update_metadata<F, Res>(&self, f: F) -> Res
    where
        F: FnOnce(&UniFsInodeSame<T, R>) -> Res,
    {
        f(&self.inode.basic)
    }
}

impl<T: RamFsProvider + 'static, R: VfsRawMutex + 'static> VfsFile for RamFsDirInode<T, R> {
    fn readdir(&self, start_index: usize) -> VfsResult<Option<VfsDirEntry>> {
        self.inode.readdir(start_index)
    }
}

impl<T: RamFsProvider + 'static, R: VfsRawMutex + 'static> VfsInode for RamFsDirInode<T, R> {
    fn get_super_block(&self) -> VfsResult<Arc<dyn VfsSuperBlock>> {
        self.inode.get_super_block()
    }

    fn node_perm(&self) -> VfsNodePerm {
        self.inode.node_perm()
    }

    fn create(
        &self,
        name: &str,
        ty: VfsNodeType,
        perm: VfsNodePerm,
        _rdev: Option<u64>,
    ) -> VfsResult<Arc<dyn VfsInode>> {
        let sb = self
            .get_super_block()?
            .downcast_arc::<UniFsSuperBlock<R>>()
            .map_err(|_| VfsError::Invalid)?;
        let inode_number = sb
            .inode_index
            .fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        sb.inode_count
            .fetch_add(1, core::sync::atomic::Ordering::SeqCst);

        let inode: Arc<dyn VfsInode> = match ty {
            VfsNodeType::File => Arc::new(RamFsFileInode::<_, R>::new(
                &sb,
                self.inode.basic.provider.clone(),
                inode_number,
                perm,
            )),
            VfsNodeType::Dir => Arc::new(RamFsDirInode::<_, R>::new(
                &sb,
                self.inode.basic.provider.clone(),
                inode_number,
                perm,
            )),
            _ => {
                return Err(VfsError::Invalid);
            }
        };
        sb.insert_inode(inode_number, inode.clone());
        self.inode
            .children
            .lock()
            .push((name.to_string(), inode_number));
        Ok(inode)
    }
    fn link(&self, name: &str, src: Arc<dyn VfsInode>) -> VfsResult<Arc<dyn VfsInode>> {
        let sb = self
            .get_super_block()?
            .downcast_arc::<UniFsSuperBlock<R>>()
            .map_err(|_| VfsError::Invalid)?;
        sb.inode_count
            .fetch_add(1, core::sync::atomic::Ordering::SeqCst);

        let inode = src
            .downcast_arc::<RamFsFileInode<T, R>>()
            .map_err(|_| VfsError::Invalid)?;

        let inode_number = inode.update_metadata(|meta| {
            meta.inner.lock().link_count += 1;
            meta.inode_number
        });
        self.inode
            .children
            .lock()
            .push((name.to_string(), inode_number));

        Ok(inode)
    }

    fn unlink(&self, name: &str) -> VfsResult<()> {
        let sb = self
            .get_super_block()?
            .downcast_arc::<UniFsSuperBlock<R>>()
            .map_err(|_| VfsError::Invalid)?;
        let index = self
            .inode
            .children
            .lock()
            .iter()
            .position(|(n, _)| n == name)
            .ok_or(VfsError::NoEntry)?;
        let (_, inode_number) = self.inode.children.lock().get(index).unwrap().clone();
        let inode = sb.get_inode(inode_number).unwrap();

        macro_rules! gen {
            ($name:ident) => {{
                let inode = inode
                    .downcast_arc::<$name<T, R>>()
                    .map_err(|_| VfsError::Invalid)?;
                let res = inode.update_metadata(|meta| {
                    meta.inner.lock().link_count -= 1;
                    meta.inner.lock().link_count
                });
                res
            }};
        }
        let link_count = if inode.inode_type() == VfsNodeType::File {
            gen!(RamFsFileInode)
        } else if inode.inode_type() == VfsNodeType::SymLink {
            gen!(RamFsSymLinkInode)
        } else {
            return Err(VfsError::Invalid);
        };

        if link_count == 0 {
            sb.inode_count
                .fetch_sub(1, core::sync::atomic::Ordering::SeqCst);
            sb.remove_inode(inode_number);
        } // delete inode from sb
        self.inode.children.lock().remove(index);
        Ok(())
    }

    fn symlink(&self, name: &str, sy_name: &str) -> VfsResult<Arc<dyn VfsInode>> {
        let sb = self
            .get_super_block()?
            .downcast_arc::<UniFsSuperBlock<R>>()
            .map_err(|_| VfsError::Invalid)?;
        let inode_number = sb
            .inode_index
            .fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        sb.inode_count
            .fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        let inode = Arc::new(RamFsSymLinkInode::<_, R>::new(
            &sb,
            self.inode.basic.provider.clone(),
            inode_number,
            sy_name.to_string(),
        ));
        self.inode
            .children
            .lock()
            .push((name.to_string(), inode_number));
        sb.insert_inode(inode_number, inode.clone());
        Ok(inode)
    }
    fn lookup(&self, name: &str) -> VfsResult<Arc<dyn VfsInode>> {
        self.inode.lookup(name)
    }

    fn rmdir(&self, name: &str) -> VfsResult<()> {
        self.unlink(name)
    }

    fn set_attr(&self, attr: InodeAttr) -> VfsResult<()> {
        set_attr(&self.inode.basic, attr);
        Ok(())
    }
    fn get_attr(&self) -> VfsResult<VfsFileStat> {
        let mut stat = basic_file_stat(&self.inode.basic);
        stat.st_size = 4096;
        stat.st_mode = VfsInodeMode::from(
            VfsNodePerm::from_bits_truncate(stat.st_mode as u16),
            VfsNodeType::Dir,
        )
        .bits();
        Ok(stat)
    }
    fn list_xattr(&self) -> VfsResult<Vec<String>> {
        let res = self.ext_attr.lock().keys().cloned().collect();
        Ok(res)
    }

    fn inode_type(&self) -> VfsNodeType {
        VfsNodeType::Dir
    }
    fn rename_to(
        &self,
        old_name: &str,
        new_parent: Arc<dyn VfsInode>,
        new_name: &str,
        flag: VfsRenameFlag,
    ) -> VfsResult<()> {
        let new_parent = new_parent
            .downcast_arc::<RamFsDirInode<T, R>>()
            .map_err(|_| VfsError::Invalid)?;
        self.inode
            .rename_to(old_name, &new_parent.inode, new_name, flag)
    }

    impl_dir_inode_default!();

    fn update_time(&self, time: VfsTime, now: VfsTimeSpec) -> VfsResult<()> {
        self.inode.update_time(time, now)
    }
}
