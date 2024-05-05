use alloc::{string::String, sync::Arc};
use core::cmp::min;

use vfscore::{
    error::VfsError,
    file::VfsFile,
    fstype::FileSystemFlags,
    inode::{InodeAttr, VfsInode},
    superblock::VfsSuperBlock,
    utils::{VfsFileStat, VfsNodePerm, VfsNodeType},
    VfsResult,
};

use crate::FS;

pub struct SystemSupportFS;

impl SystemSupportFS {
    pub fn new() -> Self {
        Self
    }
    pub fn serialize(&self) -> String {
        let mut res = String::new();
        let fs = FS.lock();
        for (_, fs) in fs.iter() {
            let flag = fs.fs_flag();
            if !flag.contains(FileSystemFlags::REQUIRES_DEV) {
                res.push_str("nodev ")
            } else {
                res.push_str("      ");
            }
            res.push_str(fs.fs_name());
            res.push_str("\n");
        }
        res
    }
}

impl VfsFile for SystemSupportFS {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let info = self.serialize();
        let min_len = min(buf.len(), info.as_bytes().len() - offset as usize);
        buf[..min_len].copy_from_slice(&info.as_bytes()[..min_len]);
        Ok(min_len)
    }
}

impl VfsInode for SystemSupportFS {
    fn get_super_block(&self) -> VfsResult<Arc<dyn VfsSuperBlock>> {
        Err(VfsError::NoSys)
    }

    fn node_perm(&self) -> VfsNodePerm {
        VfsNodePerm::empty()
    }

    fn set_attr(&self, _attr: InodeAttr) -> VfsResult<()> {
        Ok(())
    }

    fn get_attr(&self) -> VfsResult<VfsFileStat> {
        Ok(VfsFileStat {
            st_size: self.serialize().as_bytes().len() as u64,
            ..Default::default()
        })
    }

    fn inode_type(&self) -> VfsNodeType {
        VfsNodeType::File
    }
}
