use crate::Basic;
use constants::io::FileStat;
use rref::{RRef, RRefVec, RpcResult};

pub type InodeId = u64;
pub const VFS_ROOT_ID: InodeId = 0;
pub const VFS_STDIN_ID: InodeId = 1;
pub const VFS_STDOUT_ID: InodeId = 2;
pub const VFS_STDERR_ID: InodeId = 3;

pub trait VfsDomain: Basic {
    fn vfs_open(
        &self,
        root: InodeId,
        path: &RRefVec<u8>,
        mode: u32,
        open_flags: usize,
    ) -> RpcResult<InodeId>;
    fn vfs_getattr(&self, inode: InodeId, attr: RRef<FileStat>) -> RpcResult<RRef<FileStat>>;
    fn vfs_read_at(
        &self,
        inode: InodeId,
        offset: u64,
        buf: RRefVec<u8>,
    ) -> RpcResult<(RRefVec<u8>, usize)>;

    fn vfs_read(&self, inode: InodeId, buf: RRefVec<u8>) -> RpcResult<(RRefVec<u8>, usize)>;

    fn vfs_write_at(
        &self,
        inode: InodeId,
        offset: u64,
        buf: RRefVec<u8>,
    ) -> RpcResult<(RRefVec<u8>, usize)>;
    fn vfs_write(&self, inode: InodeId, buf: &RRefVec<u8>) -> RpcResult<usize>;
}
