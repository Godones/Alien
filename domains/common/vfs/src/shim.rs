use alloc::{
    string::{String, ToString},
    sync::{Arc, Weak},
    vec::Vec,
};

use interface::{DirEntryWrapper, FsDomain, InodeID};
use ksync::Mutex;
use rref::{RRef, RRefVec};
use unifs::dentry::UniFsDentry;
use vfscore::{
    dentry::VfsDentry,
    error::VfsError,
    file::VfsFile,
    fstype::{FileSystemFlags, VfsFsType, VfsMountPoint},
    inode::{InodeAttr, VfsInode},
    superblock::{SuperType, VfsSuperBlock},
    utils::*,
    VfsResult,
};

pub struct RootShimDentry {
    dentry: Arc<UniFsDentry<Mutex<()>>>,
}

impl RootShimDentry {
    pub fn new(fs_domain: Arc<dyn FsDomain>, inode_id: InodeID) -> Self {
        let fs = Arc::new(ShimFs::new(fs_domain.clone()));
        let inode = Arc::new(FsShimInode::new(fs_domain.clone(), inode_id));
        let sb = Arc::new(ShimSuperBlock::new(fs_domain, inode.clone(), fs));
        inode.set_super_block(Arc::downgrade(&sb));
        let parent = Weak::<UniFsDentry<Mutex<()>>>::new();
        let dentry = UniFsDentry::root(inode, parent);
        Self {
            dentry: Arc::new(dentry),
        }
    }
}

impl VfsDentry for RootShimDentry {
    fn name(&self) -> String {
        self.dentry.name()
    }

    fn to_mount_point(
        self: Arc<Self>,
        sub_fs_root: Arc<dyn VfsDentry>,
        mount_flag: u32,
    ) -> VfsResult<()> {
        let dentry = self.dentry.clone();
        dentry.to_mount_point(sub_fs_root, mount_flag)
    }

    fn inode(&self) -> VfsResult<Arc<dyn VfsInode>> {
        self.dentry.inode()
    }

    fn mount_point(&self) -> Option<VfsMountPoint> {
        self.dentry.mount_point()
    }

    fn clear_mount_point(&self) {
        self.dentry.clear_mount_point()
    }

    fn find(&self, path: &str) -> Option<Arc<dyn VfsDentry>> {
        self.dentry.find(path)
    }

    fn insert(
        self: Arc<Self>,
        name: &str,
        child: Arc<dyn VfsInode>,
    ) -> VfsResult<Arc<dyn VfsDentry>> {
        let dentry = self.dentry.clone();
        dentry.insert(name, child)
    }

    fn remove(&self, name: &str) -> Option<Arc<dyn VfsDentry>> {
        self.dentry.remove(name)
    }

    fn parent(&self) -> Option<Arc<dyn VfsDentry>> {
        self.dentry.parent()
    }

    fn set_parent(&self, parent: &Arc<dyn VfsDentry>) {
        self.dentry.set_parent(parent)
    }
}
struct FsShimInode {
    ino: InodeID,
    fs_domain: Arc<dyn FsDomain>,
    sb: Mutex<Option<Weak<dyn VfsSuperBlock>>>,
}

impl Drop for FsShimInode {
    fn drop(&mut self) {
        self.fs_domain.drop_inode(self.ino).unwrap();
    }
}

impl FsShimInode {
    pub fn new(fs_domain: Arc<dyn FsDomain>, ino: InodeID) -> Self {
        Self {
            fs_domain,
            ino,
            sb: Mutex::new(None),
        }
    }
    pub fn set_super_block(&self, sb: Weak<ShimSuperBlock>) {
        *self.sb.lock() = Some(sb);
    }
}

impl VfsFile for FsShimInode {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let shared_buf = RRefVec::new(0, buf.len());
        let (shared_buf, len) = self.fs_domain.read_at(self.ino, offset, shared_buf)?;
        buf[..len].copy_from_slice(&shared_buf.as_slice()[..len]);
        Ok(len)
    }
    fn write_at(&self, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        let shared_buf = RRefVec::from_slice(buf);
        let len = self.fs_domain.write_at(self.ino, offset, &shared_buf)?;
        Ok(len)
    }
    fn readdir(&self, start_index: usize) -> VfsResult<Option<VfsDirEntry>> {
        // todo!(fix name len)
        let shared_name = RRefVec::new(0, 64);
        let dir_entry = RRef::new(DirEntryWrapper::new(shared_name));
        let dir_entry = self.fs_domain.readdir(self.ino, start_index, dir_entry)?;
        if dir_entry.name_len == 0 {
            Ok(None)
        } else {
            let name = core::str::from_utf8(&dir_entry.name.as_slice()[..dir_entry.name_len])
                .unwrap()
                .to_string();
            Ok(Some(VfsDirEntry {
                ino: dir_entry.ino,
                name,
                ty: dir_entry.ty,
            }))
        }
    }
    fn poll(&self, event: VfsPollEvents) -> VfsResult<VfsPollEvents> {
        let event = self.fs_domain.poll(self.ino, event)?;
        Ok(event)
    }
    fn ioctl(&self, cmd: u32, arg: usize) -> VfsResult<usize> {
        let res = self.fs_domain.ioctl(self.ino, cmd, arg)?;
        Ok(res)
    }
    fn flush(&self) -> VfsResult<()> {
        self.fs_domain.flush(self.ino)?;
        Ok(())
    }
    fn fsync(&self) -> VfsResult<()> {
        self.fs_domain.fsync(self.ino)?;
        Ok(())
    }
}

impl VfsInode for FsShimInode {
    fn get_super_block(&self) -> VfsResult<Arc<dyn VfsSuperBlock>> {
        self.sb
            .lock()
            .as_ref()
            .unwrap()
            .upgrade()
            .ok_or(VfsError::Invalid)
    }
    fn node_perm(&self) -> VfsNodePerm {
        let perm = self.fs_domain.node_permission(self.ino).unwrap();
        perm
    }
    fn create(
        &self,
        name: &str,
        ty: VfsNodeType,
        perm: VfsNodePerm,
        rdev: Option<u64>,
    ) -> VfsResult<Arc<dyn VfsInode>> {
        let shared_name = RRefVec::from_slice(name.as_bytes());
        let inode_id = self
            .fs_domain
            .create(self.ino, &shared_name, ty, perm, rdev)?;
        let inode = Arc::new(FsShimInode::new(self.fs_domain.clone(), inode_id));
        Ok(inode)
    }
    fn link(&self, name: &str, src: Arc<dyn VfsInode>) -> VfsResult<Arc<dyn VfsInode>> {
        let shared_name = RRefVec::from_slice(name.as_bytes());
        let src = src
            .downcast_arc::<FsShimInode>()
            .map_err(|_| VfsError::Invalid)?;
        let inode_id = self.fs_domain.link(self.ino, &shared_name, src.ino)?;
        let inode = Arc::new(FsShimInode::new(self.fs_domain.clone(), inode_id));
        Ok(inode)
    }
    fn unlink(&self, name: &str) -> VfsResult<()> {
        let shared_name = RRefVec::from_slice(name.as_bytes());
        self.fs_domain.unlink(self.ino, &shared_name)?;
        Ok(())
    }
    fn symlink(&self, name: &str, sy_name: &str) -> VfsResult<Arc<dyn VfsInode>> {
        let shared_name = RRefVec::from_slice(name.as_bytes());
        let shared_sy_name = RRefVec::from_slice(sy_name.as_bytes());
        let inode_id = self
            .fs_domain
            .symlink(self.ino, &shared_name, &shared_sy_name)?;
        let inode = Arc::new(FsShimInode::new(self.fs_domain.clone(), inode_id));
        Ok(inode)
    }
    fn lookup(&self, name: &str) -> VfsResult<Arc<dyn VfsInode>> {
        let shared_name = RRefVec::from_slice(name.as_bytes());
        let inode_id = self.fs_domain.lookup(self.ino, &shared_name)?;
        let inode = Arc::new(FsShimInode::new(self.fs_domain.clone(), inode_id));
        Ok(inode)
    }

    fn rmdir(&self, name: &str) -> VfsResult<()> {
        let shared_name = RRefVec::from_slice(name.as_bytes());
        self.fs_domain.rmdir(self.ino, &shared_name)?;
        Ok(())
    }
    fn readlink(&self, buf: &mut [u8]) -> VfsResult<usize> {
        let shared_buf = RRefVec::new(0, buf.len());
        let (shared_buf, len) = self.fs_domain.readlink(self.ino, shared_buf)?;
        buf[..len].copy_from_slice(&shared_buf.as_slice()[..len]);
        Ok(len)
    }
    fn set_attr(&self, attr: InodeAttr) -> VfsResult<()> {
        self.fs_domain.set_attr(self.ino, attr)?;
        Ok(())
    }
    fn get_attr(&self) -> VfsResult<VfsFileStat> {
        let attr = self.fs_domain.get_attr(self.ino)?;
        Ok(attr)
    }
    fn list_xattr(&self) -> VfsResult<Vec<String>> {
        panic!("We should not call this function now");
    }
    fn inode_type(&self) -> VfsNodeType {
        self.fs_domain.inode_type(self.ino).unwrap()
    }
    fn truncate(&self, len: u64) -> VfsResult<()> {
        self.fs_domain.truncate(self.ino, len)?;
        Ok(())
    }
    fn rename_to(
        &self,
        old_name: &str,
        new_parent: Arc<dyn VfsInode>,
        new_name: &str,
        flag: VfsRenameFlag,
    ) -> VfsResult<()> {
        let shared_old_name = RRefVec::from_slice(old_name.as_bytes());
        let shared_new_name = RRefVec::from_slice(new_name.as_bytes());
        let new_parent = new_parent
            .downcast_arc::<FsShimInode>()
            .map_err(|_| VfsError::Invalid)?;
        self.fs_domain.rename(
            self.ino,
            &shared_old_name,
            new_parent.ino,
            &shared_new_name,
            flag,
        )?;
        Ok(())
    }
    fn update_time(&self, time: VfsTime, now: VfsTimeSpec) -> VfsResult<()> {
        self.fs_domain.update_time(self.ino, time, now)?;
        Ok(())
    }
}
struct ShimSuperBlock {
    fs_domain: Arc<dyn FsDomain>,
    root_inode: Arc<dyn VfsInode>,
    fs: Arc<ShimFs>,
}

impl ShimSuperBlock {
    pub fn new(
        fs_domain: Arc<dyn FsDomain>,
        root_inode: Arc<dyn VfsInode>,
        fs: Arc<ShimFs>,
    ) -> Self {
        Self {
            fs_domain,
            root_inode,
            fs,
        }
    }
}

impl VfsSuperBlock for ShimSuperBlock {
    fn sync_fs(&self, wait: bool) -> VfsResult<()> {
        self.fs_domain.sync_fs(wait)?;
        Ok(())
    }

    fn stat_fs(&self) -> VfsResult<VfsFsStat> {
        let fs_stat = RRef::new(VfsFsStat::default());
        let fs_stat = self.fs_domain.stat_fs(fs_stat)?;
        Ok(*fs_stat)
    }

    fn super_type(&self) -> SuperType {
        let ty = self.fs_domain.super_type().unwrap();
        ty
    }

    fn fs_type(&self) -> Arc<dyn VfsFsType> {
        self.fs.clone()
    }

    fn root_inode(&self) -> VfsResult<Arc<dyn VfsInode>> {
        Ok(self.root_inode.clone())
    }
}
struct ShimFs {
    fs_domain: Arc<dyn FsDomain>,
}

impl ShimFs {
    pub fn new(fs_domain: Arc<dyn FsDomain>) -> Self {
        Self { fs_domain }
    }
}

impl VfsFsType for ShimFs {
    fn mount(
        self: Arc<Self>,
        _flags: u32,
        _ab_mnt: &str,
        _dev: Option<Arc<dyn VfsInode>>,
        _data: &[u8],
    ) -> VfsResult<Arc<dyn VfsDentry>> {
        panic!("We should call this function")
    }

    fn kill_sb(&self, _sb: Arc<dyn VfsSuperBlock>) -> VfsResult<()> {
        self.fs_domain.kill_sb()?;
        Ok(())
    }

    fn fs_flag(&self) -> FileSystemFlags {
        let flag = self.fs_domain.fs_flag().unwrap();
        flag
    }

    fn fs_name(&self) -> String {
        let buf = RRefVec::new(0, 16);
        let (buf, len) = self.fs_domain.fs_name(buf).unwrap();
        core::str::from_utf8(&buf.as_slice()[..len])
            .unwrap()
            .to_string()
    }
}
