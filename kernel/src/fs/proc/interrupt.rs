use crate::config::INTERRUPT_RECORD;
use alloc::sync::Arc;
use core::cmp::min;
use vfscore::error::VfsError;
use vfscore::file::VfsFile;
use vfscore::inode::{InodeAttr, VfsInode};
use vfscore::superblock::VfsSuperBlock;
use vfscore::utils::{FileStat, VfsNodeType};
use vfscore::VfsResult;

pub struct InterruptRecord;

impl VfsFile for InterruptRecord {
    fn read_at(&self, _offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let min_len = min(buf.len(), INTERRUPT_RECORD.as_bytes().len());
        buf[..min_len].copy_from_slice(&INTERRUPT_RECORD.as_bytes()[..min_len]);
        Ok(min_len)
    }
}

impl VfsInode for InterruptRecord {
    fn get_super_block(&self) -> VfsResult<Arc<dyn VfsSuperBlock>> {
        Err(VfsError::NoSys)
    }

    fn set_attr(&self, _attr: InodeAttr) -> VfsResult<()> {
        Ok(())
    }

    fn get_attr(&self) -> VfsResult<FileStat> {
        Ok(FileStat {
            st_size: INTERRUPT_RECORD.as_bytes().len() as u64,
            ..Default::default()
        })
    }

    fn inode_type(&self) -> VfsNodeType {
        VfsNodeType::File
    }
}
