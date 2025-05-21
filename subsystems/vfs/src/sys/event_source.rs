use vfscore::{file::VfsFile, inode::VfsInode, utils::VfsNodeType, VfsResult};

#[derive(Debug)]
pub struct KprobeType;

impl VfsFile for KprobeType {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        if offset != 0 {
            return Ok(0);
        }
        // perf_type_id::PERF_TYPE_MAX
        buf[0] = b'6';
        Ok(1)
    }
}
impl VfsInode for KprobeType {
    fn inode_type(&self) -> VfsNodeType {
        VfsNodeType::File
    }
}
