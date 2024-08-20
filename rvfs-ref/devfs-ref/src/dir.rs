use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::ops::Deref;

use unifs::inode::UniFsDirInode;
use vfscore::{
    error::VfsError,
    file::VfsFile,
    impl_dir_inode_default,
    inode::InodeAttr,
    utils::{
        VfsDirEntry, VfsFileStat, VfsInodeMode, VfsNodePerm, VfsNodeType, VfsRenameFlag, VfsTime,
    },
    RRefVec,
};

use crate::{dev::DevFsDevInode, *};

pub struct DevFsDirInode<T: Send + Sync, R: VfsRawMutex>(UniFsDirInode<T, R>);

impl<T: DevKernelProvider + 'static, R: VfsRawMutex + 'static> DevFsDirInode<T, R> {
    pub fn new(
        inode_number: u64,
        provider: T,
        sb: &Arc<UniFsSuperBlock<R>>,
        perm: VfsNodePerm,
    ) -> Self {
        Self(UniFsDirInode {
            basic: UniFsInodeSame::new(sb, provider, inode_number, perm),
            children: lock_api::Mutex::new(Vec::new()),
        })
    }
}

impl<T: Send + Sync + 'static, R: VfsRawMutex + 'static> Deref for DevFsDirInode<T, R> {
    type Target = UniFsDirInode<T, R>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Send + Sync + 'static, R: VfsRawMutex + 'static> VfsFile for DevFsDirInode<T, R> {
    fn readdir(&self, start_index: usize) -> VfsResult<Option<VfsDirEntry>> {
        self.0.readdir(start_index)
    }
}

impl<T: DevKernelProvider + 'static, R: VfsRawMutex + 'static> VfsInode for DevFsDirInode<T, R> {
    fn get_super_block(&self) -> VfsResult<Arc<dyn VfsSuperBlock>> {
        self.0.get_super_block()
    }

    fn node_perm(&self) -> VfsNodePerm {
        self.0.node_perm()
    }

    fn create(
        &self,
        name: &str,
        ty: VfsNodeType,
        perm: VfsNodePerm,
        rdev: Option<u64>,
    ) -> VfsResult<Arc<dyn VfsInode>> {
        if ty != VfsNodeType::Dir && rdev.is_none() {
            return Err(VfsError::Invalid);
        }
        let sb = self.basic.sb.upgrade().unwrap();
        let inode_number = sb
            .inode_index
            .fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        sb.inode_count
            .fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        let inode: Arc<dyn VfsInode> = match ty {
            VfsNodeType::Dir => Arc::new(DevFsDirInode::<T, R>::new(
                inode_number,
                self.basic.provider.clone(),
                &sb,
                perm,
            )),
            VfsNodeType::BlockDevice
            | VfsNodeType::CharDevice
            | VfsNodeType::Fifo
            | VfsNodeType::Socket => Arc::new(DevFsDevInode::new(
                &sb,
                self.basic.provider.clone(),
                inode_number,
                rdev.unwrap(),
                ty,
            )),
            _ => {
                return Err(VfsError::Invalid);
            }
        };
        sb.insert_inode(inode_number, inode.clone());
        self.children.lock().push((name.to_string(), inode_number));
        Ok(inode)
    }

    fn link(&self, _name: &str, _src: Arc<dyn VfsInode>) -> VfsResult<Arc<dyn VfsInode>> {
        Err(VfsError::NoSys)
    }

    fn unlink(&self, name: &str) -> VfsResult<()> {
        let sb = self.basic.sb.upgrade().unwrap();
        let mut children = self.children.lock();
        children
            .iter()
            .position(|(n, _)| n == name)
            .map_or(Err(VfsError::NoEntry), |index| {
                let (_, inode_number) = children.remove(index);
                sb.remove_inode(inode_number);
                Ok(())
            })?;
        Ok(())
    }
    fn symlink(&self, _name: &str, _sy_name: &str) -> VfsResult<Arc<dyn VfsInode>> {
        Err(VfsError::NoSys)
    }

    fn lookup(&self, name: &str) -> VfsResult<Arc<dyn VfsInode>> {
        self.0.lookup(name)
    }

    fn rmdir(&self, name: &str) -> VfsResult<()> {
        self.unlink(name)
    }

    fn set_attr(&self, attr: InodeAttr) -> VfsResult<()> {
        self.0.set_attr(attr)
    }

    impl_dir_inode_default!();

    fn get_attr(&self) -> VfsResult<VfsFileStat> {
        self.0.get_attr().map(|mut attr| {
            attr.st_mode = VfsInodeMode::from(
                VfsNodePerm::from_bits_truncate(attr.st_mode as u16),
                VfsNodeType::Dir,
            )
            .bits();
            attr
        })
    }

    fn list_xattr(&self) -> VfsResult<Vec<String>> {
        Err(VfsError::NoSys)
    }

    fn inode_type(&self) -> VfsNodeType {
        self.0.inode_type()
    }

    fn rename_to(
        &self,
        old_name: &str,
        new_parent: Arc<dyn VfsInode>,
        new_name: &str,
        flag: VfsRenameFlag,
    ) -> VfsResult<()> {
        let new_parent = new_parent
            .downcast_arc::<DevFsDirInode<T, R>>()
            .map_err(|_| VfsError::Invalid)?;
        self.0.rename_to(old_name, &new_parent.0, new_name, flag)
    }

    fn update_time(&self, time: VfsTime, now: VfsTimeSpec) -> VfsResult<()> {
        self.0.update_time(time, now)
    }
}
