use crate::config::MEMINFO;
use alloc::sync::Arc;
use core::cmp::min;
use vfscore::error::VfsError;
use vfscore::file::VfsFile;
use vfscore::inode::{InodeAttr, VfsInode};
use vfscore::superblock::VfsSuperBlock;
use vfscore::utils::{VfsFileStat, VfsNodePerm, VfsNodeType};
use vfscore::VfsResult;

pub struct MemInfo;

impl VfsFile for MemInfo {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let min_len = min(buf.len(), MEMINFO.as_bytes().len() - offset as usize);
        buf[..min_len].copy_from_slice(&MEMINFO.as_bytes()[..min_len]);
        Ok(min_len)
    }
}

impl VfsInode for MemInfo {
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
            st_size: MEMINFO.as_bytes().len() as u64,
            ..Default::default()
        })
    }

    fn inode_type(&self) -> VfsNodeType {
        VfsNodeType::File
    }
}
