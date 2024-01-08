use crate::interrupt::record::interrupts_info;
use alloc::sync::Arc;
use core::cmp::min;
use vfscore::error::VfsError;
use vfscore::file::VfsFile;
use vfscore::inode::{InodeAttr, VfsInode};
use vfscore::superblock::VfsSuperBlock;
use vfscore::utils::{VfsFileStat, VfsNodePerm, VfsNodeType};
use vfscore::VfsResult;

pub struct InterruptRecord;

impl VfsFile for InterruptRecord {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let info = interrupts_info();
        let min_len = min(buf.len(), info.as_bytes().len() - offset as usize);
        buf[..min_len].copy_from_slice(&info.as_bytes()[..min_len]);
        Ok(min_len)
    }
    fn write_at(&self, _offset: u64, _buf: &[u8]) -> VfsResult<usize> {
        Err(VfsError::PermissionDenied)
    }
}

impl VfsInode for InterruptRecord {
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
        let info = interrupts_info();
        Ok(VfsFileStat {
            st_size: info.as_bytes().len() as u64,
            ..Default::default()
        })
    }

    fn inode_type(&self) -> VfsNodeType {
        VfsNodeType::File
    }
}
