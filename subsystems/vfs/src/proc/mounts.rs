use alloc::sync::Arc;
use core::cmp::min;

use vfscore::{
    error::VfsError,
    file::VfsFile,
    inode::{InodeAttr, VfsInode},
    superblock::VfsSuperBlock,
    utils::{VfsFileStat, VfsNodePerm, VfsNodeType},
    VfsResult,
};

// todo!(dynamic mount info)
const MOUNT_INFO: &str = r"
 rootfs / rootfs rw 0 0
 devfs /dev devfs rw 0 0
 fat32 / fat rw 0 0
";
pub struct MountInfo;

impl VfsFile for MountInfo {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let min_len = min(buf.len(), MOUNT_INFO.as_bytes().len() - offset as usize);
        buf[..min_len].copy_from_slice(&MOUNT_INFO.as_bytes()[..min_len]);
        Ok(min_len)
    }
}

impl VfsInode for MountInfo {
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
            st_size: MOUNT_INFO.as_bytes().len() as u64,
            ..Default::default()
        })
    }

    fn inode_type(&self) -> VfsNodeType {
        VfsNodeType::File
    }
}
