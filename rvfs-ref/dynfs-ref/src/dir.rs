use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use unifs::inode::UniFsDirInode;
use vfscore::{
    error::VfsError,
    file::VfsFile,
    impl_dir_inode_default,
    inode::InodeAttr,
    utils::{
        VfsDirEntry, VfsFileStat, VfsInodeMode, VfsNodePerm, VfsNodeType, VfsRenameFlag, VfsTime,
    },
    DVec,
};

use crate::{file::DynFsFileInode, *};

pub struct DynFsDirInode<T: Send + Sync, R: VfsRawMutex>(UniFsDirInode<T, R>);

impl<T: DynFsKernelProvider + 'static, R: VfsRawMutex + 'static> DynFsDirInode<T, R> {
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

    fn add_manually(
        &self,
        ty: VfsNodeType,
        name: &str,
        inode: Option<Arc<dyn VfsInode>>,
        perm: VfsNodePerm,
    ) -> VfsResult<Arc<dyn VfsInode>> {
        let sb = self.0.basic.sb.upgrade().unwrap();
        let inode_number = sb
            .inode_index
            .fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        sb.inode_count
            .fetch_add(1, core::sync::atomic::Ordering::SeqCst);

        let res: Arc<dyn VfsInode> = match ty {
            VfsNodeType::File => Arc::new(DynFsFileInode::new(
                &sb,
                self.0.basic.provider.clone(),
                inode_number,
                inode.unwrap(),
                perm,
            )) as _,
            VfsNodeType::Dir => Arc::new(DynFsDirInode::new(
                inode_number,
                self.0.basic.provider.clone(),
                &sb,
                perm,
            )),
            _ => return Err(VfsError::NoSys),
        };
        sb.insert_inode(inode_number, res.clone());
        self.0
            .children
            .lock()
            .push((name.to_string(), inode_number));
        Ok(res)
    }
    pub fn add_file_manually(
        &self,
        name: &str,
        inode: Arc<dyn VfsInode>,
        perm: VfsNodePerm,
    ) -> VfsResult<Arc<dyn VfsInode>> {
        self.add_manually(VfsNodeType::File, name, Some(inode), perm)
    }

    pub fn add_dir_manually(&self, name: &str, perm: VfsNodePerm) -> VfsResult<Arc<dyn VfsInode>> {
        self.add_manually(VfsNodeType::Dir, name, None, perm)
    }

    pub fn remove_manually(&self, name: &str) -> VfsResult<()> {
        let mut children = self.0.children.lock();
        let index = children
            .iter()
            .position(|(n, _)| n == name)
            .ok_or(VfsError::NoEntry)?;
        let (_, inode_number) = children.remove(index);
        let sb = self.0.basic.sb.upgrade().unwrap();
        sb.remove_inode(inode_number);
        sb.inode_count
            .fetch_sub(1, core::sync::atomic::Ordering::SeqCst);
        Ok(())
    }
}

impl<T: Send + Sync + 'static, R: VfsRawMutex + 'static> VfsFile for DynFsDirInode<T, R> {
    fn readdir(&self, start_index: usize) -> VfsResult<Option<VfsDirEntry>> {
        self.0.readdir(start_index)
    }
}

impl<T: DynFsKernelProvider + 'static, R: VfsRawMutex + 'static> VfsInode for DynFsDirInode<T, R> {
    fn get_super_block(&self) -> VfsResult<Arc<dyn VfsSuperBlock>> {
        self.0.get_super_block()
    }

    fn node_perm(&self) -> VfsNodePerm {
        self.0.node_perm()
    }

    fn create(
        &self,
        _name: &str,
        _ty: VfsNodeType,
        _perm: VfsNodePerm,
        _rdev: Option<u64>,
    ) -> VfsResult<Arc<dyn VfsInode>> {
        Err(VfsError::NoSys)
    }

    fn link(&self, _name: &str, _src: Arc<dyn VfsInode>) -> VfsResult<Arc<dyn VfsInode>> {
        Err(VfsError::NoSys)
    }

    fn unlink(&self, _name: &str) -> VfsResult<()> {
        Err(VfsError::NoSys)
    }

    fn symlink(&self, _name: &str, _sy_name: &str) -> VfsResult<Arc<dyn VfsInode>> {
        Err(VfsError::NoSys)
    }

    fn lookup(&self, name: &str) -> VfsResult<Arc<dyn VfsInode>> {
        self.0.lookup(name)
    }
    fn rmdir(&self, _name: &str) -> VfsResult<()> {
        Err(VfsError::NoSys)
    }

    impl_dir_inode_default!();

    fn set_attr(&self, attr: InodeAttr) -> VfsResult<()> {
        self.0.set_attr(attr)
    }

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
        _old_name: &str,
        _new_parent: Arc<dyn VfsInode>,
        _new_name: &str,
        _flag: VfsRenameFlag,
    ) -> VfsResult<()> {
        Err(VfsError::NoSys)
    }

    fn update_time(&self, time: VfsTime, now: VfsTimeSpec) -> VfsResult<()> {
        self.0.update_time(time, now)
    }
}
