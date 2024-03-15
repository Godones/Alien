#![no_std]
// #![deny(unsafe_code)]
extern crate alloc;
extern crate malloc;

use crate::kfile::{File, KernelFile};
use crate::tree::system_root_fs;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use constants::io::{FileStat, OpenFlags};
use constants::AlienError;
use core::sync::atomic::AtomicU64;
use interface::{Basic, InodeId, VfsDomain};
use ksync::RwLock;
use log::info;
use rref::{RRef, RRefVec, RpcError, RpcResult};
use vfscore::dentry::VfsDentry;
use vfscore::path::VfsPath;
use vfscore::utils::VfsInodeMode;

mod devfs;
mod kfile;
mod pipefs;
mod procfs;
mod ramfs;
mod sys;
mod tree;

static VFS_MAP: RwLock<BTreeMap<InodeId, Arc<dyn File>>> = RwLock::new(BTreeMap::new());
static INODE_ID: AtomicU64 = AtomicU64::new(4);

fn alloc_inode_id() -> InodeId {
    INODE_ID.fetch_add(1, core::sync::atomic::Ordering::SeqCst)
}

fn insert_dentry(inode: InodeId, dentry: Arc<dyn VfsDentry>, open_flags: OpenFlags) {
    let file = KernelFile::new(dentry.clone(), open_flags);
    VFS_MAP.write().insert(inode, Arc::new(file));
}

fn remove_file(inode: InodeId) {
    VFS_MAP.write().remove(&inode);
}

fn get_file(inode: InodeId) -> Option<Arc<dyn File>> {
    VFS_MAP.read().get(&inode).map(|f| f.clone())
}

#[derive(Debug)]
struct VfsDomainImpl;

impl Basic for VfsDomainImpl {}

impl VfsDomain for VfsDomainImpl {
    fn vfs_open(
        &self,
        root: InodeId,
        path: &RRefVec<u8>,
        mode: u32,
        open_flags: usize,
    ) -> RpcResult<InodeId> {
        let start = get_file(root).ok_or(RpcError::Alien(AlienError::EINVAL))?;
        let root = system_root_fs();
        let path = core::str::from_utf8(path.as_slice()).unwrap();
        let mode = if mode == 0 {
            None
        } else {
            Some(VfsInodeMode::from_bits_truncate(mode))
        };

        info!("vfs_open:  path: {:?}, mode: {:?}", path, mode);
        let open_flags = OpenFlags::from_bits_truncate(open_flags);
        let path = VfsPath::new(root, start.dentry())
            .join(path)
            .map_err(|e| AlienError::from(e))?
            .open(mode)
            .map_err(|e| AlienError::from(e))?;
        let id = alloc_inode_id();
        insert_dentry(id, path, open_flags);
        Ok(id)
    }

    fn vfs_getattr(&self, inode: InodeId, mut attr: RRef<FileStat>) -> RpcResult<RRef<FileStat>> {
        let dentry = get_file(inode).unwrap().dentry();
        let vfs_attr = dentry
            .inode()
            .map_err(|e| AlienError::from(e))?
            .get_attr()
            .map_err(|e| AlienError::from(e))?;
        let mut file_attr = FileStat::default();
        file_attr.st_dev = vfs_attr.st_dev;
        file_attr.st_ino = vfs_attr.st_ino;
        file_attr.st_mode = vfs_attr.st_mode;
        file_attr.st_nlink = vfs_attr.st_nlink;
        file_attr.st_uid = vfs_attr.st_uid;
        file_attr.st_gid = vfs_attr.st_gid;
        file_attr.st_rdev = vfs_attr.st_rdev;
        file_attr.st_size = vfs_attr.st_size;
        file_attr.st_blksize = vfs_attr.st_blksize;
        file_attr.st_blocks = vfs_attr.st_blocks;
        file_attr.st_atime_sec = vfs_attr.st_atime.sec;
        file_attr.st_atime_nsec = vfs_attr.st_atime.nsec;
        file_attr.st_mtime_sec = vfs_attr.st_mtime.sec;
        file_attr.st_mtime_nsec = vfs_attr.st_mtime.nsec;
        file_attr.st_ctime_sec = vfs_attr.st_ctime.sec;
        file_attr.st_ctime_nsec = vfs_attr.st_ctime.nsec;
        *attr = file_attr;
        Ok(attr)
    }
    fn vfs_read_at(
        &self,
        inode: InodeId,
        offset: u64,
        mut buf: RRefVec<u8>,
    ) -> RpcResult<(RRefVec<u8>, usize)> {
        let file = get_file(inode).unwrap();
        let res = file
            .read_at(offset, buf.as_mut_slice())
            .map_err(|e| AlienError::from(e))?;
        Ok((buf, res))
    }

    fn vfs_read(&self, inode: InodeId, mut buf: RRefVec<u8>) -> RpcResult<(RRefVec<u8>, usize)> {
        let file = get_file(inode).unwrap();
        let res = file
            .read(buf.as_mut_slice())
            .map_err(|e| AlienError::from(e))?;
        Ok((buf, res))
    }
    fn vfs_write_at(
        &self,
        inode: InodeId,
        offset: u64,
        buf: RRefVec<u8>,
    ) -> RpcResult<(RRefVec<u8>, usize)> {
        let file = get_file(inode).unwrap();
        let res = file
            .write_at(offset, buf.as_slice())
            .map_err(|e| AlienError::from(e))?;
        Ok((buf, res))
    }
    fn vfs_write(&self, inode: InodeId, buf: &RRefVec<u8>) -> RpcResult<usize> {
        let file = get_file(inode).unwrap();
        let res = file
            .write(buf.as_slice())
            .map_err(|e| AlienError::from(e))?;
        Ok(res)
    }
}

pub fn main() -> Arc<dyn VfsDomain> {
    tree::init_filesystem().unwrap();
    Arc::new(VfsDomainImpl)
}
