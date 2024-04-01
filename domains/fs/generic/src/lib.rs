#![no_std]
#![forbid(unsafe_code)]

mod shim;

extern crate alloc;
use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    sync::Arc,
};
use core::{
    fmt::{Debug, Formatter},
    ops::Index,
    sync::atomic::AtomicU64,
};

use basic::println;
use constants::AlienResult;
use interface::{Basic, DirEntryWrapper, DomainType, FsDomain, InodeID, VfsDomain};
use ksync::Mutex;
use rref::{RRef, RRefVec};
use spin::Once;
use vfscore::{
    dentry::VfsDentry,
    fstype::{FileSystemFlags, VfsFsType},
    inode::{InodeAttr, VfsInode},
    superblock::SuperType,
    utils::{
        VfsFileStat, VfsFsStat, VfsNodePerm, VfsNodeType, VfsPollEvents, VfsRenameFlag, VfsTime,
        VfsTimeSpec,
    },
};

use crate::shim::MountDevShimInode;

pub static VFS_DOMAIN: Once<Arc<dyn VfsDomain>> = Once::new();
pub struct GenericFsDomain {
    fs: Arc<dyn VfsFsType>,
    inode_map: Mutex<BTreeMap<InodeID, Arc<dyn VfsInode>>>,
    inode_index: AtomicU64,
    root_dentry: Once<Arc<dyn VfsDentry>>,
    name: String,
}

impl Debug for GenericFsDomain {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("GenericFsDomain")
            .field("name", &self.name)
            .finish()
    }
}

impl GenericFsDomain {
    pub fn new(fs: Arc<dyn VfsFsType>, name: String) -> Self {
        Self {
            fs,
            inode_map: Mutex::new(BTreeMap::new()),
            inode_index: AtomicU64::new(0),
            root_dentry: Once::new(),
            name,
        }
    }

    pub fn root_dentry(&self) -> Arc<dyn VfsDentry> {
        self.root_dentry.get().unwrap().clone()
    }
}

impl Basic for GenericFsDomain {}

impl FsDomain for GenericFsDomain {
    fn init(&self) -> AlienResult<()> {
        let vfs_domain = basic::get_domain("vfs").unwrap();
        let vfs_domain = match vfs_domain {
            DomainType::VfsDomain(vfs_domain) => vfs_domain,
            _ => panic!("vfs domain not found"),
        };
        VFS_DOMAIN.call_once(|| vfs_domain);
        println!("{} FsDomain init", self.name);
        Ok(())
    }
    fn mount(&self, mount_point: &RRefVec<u8>, dev_inode: Option<InodeID>) -> AlienResult<InodeID> {
        let mount_point = core::str::from_utf8(mount_point.as_slice())
            .unwrap()
            .to_string();
        let dev_inode: Option<Arc<dyn VfsInode>> = match dev_inode {
            None => None,
            Some(id) => {
                let vfs_domain = VFS_DOMAIN.get().unwrap().clone();
                let shim_dev_inode = MountDevShimInode::new(id, vfs_domain);
                Some(Arc::new(shim_dev_inode))
            }
        };
        let root = self.fs.i_mount(0, &mount_point, dev_inode, &[])?;
        let inode_id = self
            .inode_index
            .fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        self.inode_map
            .lock()
            .insert(inode_id, root.inode().unwrap());
        self.root_dentry.call_once(|| root);
        assert_eq!(inode_id, 0);
        Ok(inode_id)
    }

    fn drop_inode(&self, inode: InodeID) -> AlienResult<()> {
        self.inode_map.lock().remove(&inode);
        Ok(())
    }

    fn read_at(
        &self,
        inode: InodeID,
        offset: u64,
        mut buf: RRefVec<u8>,
    ) -> AlienResult<(RRefVec<u8>, usize)> {
        let inode = self.inode_map.lock().index(&inode).clone();
        let r = inode.read_at(offset, buf.as_mut_slice())?;
        Ok((buf, r))
    }

    fn write_at(&self, inode: InodeID, offset: u64, buf: &RRefVec<u8>) -> AlienResult<usize> {
        let inode = self.inode_map.lock().index(&inode).clone();
        let w = inode.write_at(offset, buf.as_slice())?;
        Ok(w)
    }

    fn readdir(
        &self,
        inode: InodeID,
        start_index: usize,
        mut entry: RRef<DirEntryWrapper>,
    ) -> AlienResult<RRef<DirEntryWrapper>> {
        let inode = self.inode_map.lock().index(&inode).clone();
        let vfs_entry = inode.readdir(start_index)?;
        match vfs_entry {
            None => {
                entry.name_len = 0;
            }
            Some(vfs_entry) => {
                entry.name_len = vfs_entry.name.len();
                entry.ty = vfs_entry.ty;
                entry.ino = vfs_entry.ino;
                let copy_len = core::cmp::min(entry.name_len, entry.name.len());
                entry.name.as_mut_slice()[..copy_len]
                    .copy_from_slice(&vfs_entry.name.as_bytes()[..copy_len]);
            }
        }
        Ok(entry)
    }

    fn poll(&self, inode: InodeID, mask: VfsPollEvents) -> AlienResult<VfsPollEvents> {
        let inode = self.inode_map.lock().index(&inode).clone();
        let res = inode.poll(mask)?;
        Ok(res)
    }

    fn flush(&self, inode: InodeID) -> AlienResult<()> {
        let inode = self.inode_map.lock().index(&inode).clone();
        inode.flush()?;
        Ok(())
    }

    fn fsync(&self, inode: InodeID) -> AlienResult<()> {
        let inode = self.inode_map.lock().index(&inode).clone();
        inode.fsync()?;
        Ok(())
    }

    fn rmdir(&self, parent: InodeID, name: &RRefVec<u8>) -> AlienResult<()> {
        let parent = self.inode_map.lock().index(&parent).clone();
        let name = core::str::from_utf8(name.as_slice()).unwrap();
        parent.rmdir(name)?;
        Ok(())
    }

    fn node_permission(&self, inode: InodeID) -> AlienResult<VfsNodePerm> {
        let inode = self.inode_map.lock().index(&inode).clone();
        let perm = inode.node_perm();
        Ok(perm)
    }

    fn create(
        &self,
        parent: InodeID,
        name: &RRefVec<u8>,
        ty: VfsNodeType,
        perm: VfsNodePerm,
        rdev: Option<u64>,
    ) -> AlienResult<InodeID> {
        let parent = self.inode_map.lock().index(&parent).clone();
        let name = core::str::from_utf8(name.as_slice()).unwrap();
        let inode = parent.create(name, ty, perm, rdev)?;
        let inode_id = self
            .inode_index
            .fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        self.inode_map.lock().insert(inode_id, inode);
        Ok(inode_id)
    }

    fn link(&self, parent: InodeID, name: &RRefVec<u8>, src: InodeID) -> AlienResult<InodeID> {
        let parent = self.inode_map.lock().index(&parent).clone();
        let name = core::str::from_utf8(name.as_slice()).unwrap();
        let src = self.inode_map.lock().index(&src).clone();
        let inode = parent.link(name, src.clone())?;
        let inode_id = self
            .inode_index
            .fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        self.inode_map.lock().insert(inode_id, inode);
        Ok(inode_id)
    }

    fn unlink(&self, parent: InodeID, name: &RRefVec<u8>) -> AlienResult<()> {
        let parent = self.inode_map.lock().index(&parent).clone();
        let name = core::str::from_utf8(name.as_slice()).unwrap();
        parent.unlink(name)?;
        Ok(())
    }

    fn symlink(
        &self,
        parent: InodeID,
        name: &RRefVec<u8>,
        link: &RRefVec<u8>,
    ) -> AlienResult<InodeID> {
        let parent = self.inode_map.lock().index(&parent).clone();
        let name = core::str::from_utf8(name.as_slice()).unwrap();
        let link = core::str::from_utf8(link.as_slice()).unwrap();
        let inode = parent.symlink(name, link)?;
        let inode_id = self
            .inode_index
            .fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        self.inode_map.lock().insert(inode_id, inode);
        Ok(inode_id)
    }

    fn lookup(&self, parent: InodeID, name: &RRefVec<u8>) -> AlienResult<InodeID> {
        let parent = self.inode_map.lock().index(&parent).clone();
        let name = core::str::from_utf8(name.as_slice()).unwrap();
        let inode = parent.lookup(name)?;
        let inode_id = self
            .inode_index
            .fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        self.inode_map.lock().insert(inode_id, inode);
        Ok(inode_id)
    }

    fn readlink(&self, inode: InodeID, mut buf: RRefVec<u8>) -> AlienResult<(RRefVec<u8>, usize)> {
        let inode = self.inode_map.lock().index(&inode).clone();
        let l = inode.readlink(buf.as_mut_slice())?;
        Ok((buf, l))
    }

    fn set_attr(&self, inode: InodeID, attr: InodeAttr) -> AlienResult<()> {
        let inode = self.inode_map.lock().index(&inode).clone();
        inode.set_attr(attr)?;
        Ok(())
    }

    fn get_attr(&self, inode: InodeID) -> AlienResult<VfsFileStat> {
        let inode = self.inode_map.lock().index(&inode).clone();
        let stat = inode.get_attr()?;
        Ok(stat)
    }

    fn inode_type(&self, inode: InodeID) -> AlienResult<VfsNodeType> {
        let inode = self.inode_map.lock().index(&inode).clone();
        let ty = inode.inode_type();
        Ok(ty)
    }

    fn truncate(&self, inode: InodeID, len: u64) -> AlienResult<()> {
        let inode = self.inode_map.lock().index(&inode).clone();
        inode.truncate(len)?;
        Ok(())
    }

    fn rename(
        &self,
        old_parent: InodeID,
        old_name: &RRefVec<u8>,
        new_parent: InodeID,
        new_name: &RRefVec<u8>,
        flags: VfsRenameFlag,
    ) -> AlienResult<()> {
        let old_parent = self.inode_map.lock().index(&old_parent).clone();
        let old_name = core::str::from_utf8(old_name.as_slice()).unwrap();
        let new_parent = self.inode_map.lock().index(&new_parent).clone();
        let new_name = core::str::from_utf8(new_name.as_slice()).unwrap();
        old_parent.rename_to(old_name, new_parent, new_name, flags)?;
        Ok(())
    }

    fn update_time(&self, inode: InodeID, time: VfsTime, now: VfsTimeSpec) -> AlienResult<()> {
        let inode = self.inode_map.lock().index(&inode).clone();
        inode.update_time(time, now)?;
        Ok(())
    }

    fn sync_fs(&self, wait: bool) -> AlienResult<()> {
        let inode = self.inode_map.lock().index(&0).clone();
        inode.get_super_block()?.sync_fs(wait)?;
        Ok(())
    }

    fn stat_fs(&self, mut fs_stat: RRef<VfsFsStat>) -> AlienResult<RRef<VfsFsStat>> {
        let inode = self.inode_map.lock().index(&0).clone();
        let stat = inode.get_super_block()?.stat_fs()?;
        *fs_stat = stat;
        Ok(fs_stat)
    }

    fn super_type(&self) -> AlienResult<SuperType> {
        let inode = self.inode_map.lock().index(&0).clone();
        let ty = inode.get_super_block()?.super_type();
        Ok(ty)
    }

    fn kill_sb(&self) -> AlienResult<()> {
        let inode = self.inode_map.lock().index(&0).clone();
        let sb = inode.get_super_block()?;
        self.fs.kill_sb(sb)?;
        Ok(())
    }

    fn fs_flag(&self) -> AlienResult<FileSystemFlags> {
        Ok(self.fs.fs_flag())
    }

    fn fs_name(&self, mut name: RRefVec<u8>) -> AlienResult<(RRefVec<u8>, usize)> {
        let fs_name = self.fs.fs_name();
        let copy_len = core::cmp::min(name.len(), fs_name.len());
        name.as_mut_slice()[..copy_len].copy_from_slice(&fs_name.as_bytes()[..copy_len]);
        Ok((name, copy_len))
    }
}
