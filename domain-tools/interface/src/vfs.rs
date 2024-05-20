use downcast_rs::{impl_downcast, DowncastSync};
use gproxy::proxy;
use pconst::io::SeekFrom;
use rref::{RRef, RRefVec};
use vfscore::utils::{VfsFileStat, VfsNodeType, VfsPollEvents};

use super::AlienResult;
use crate::{Basic, SocketID};

pub type InodeID = u64;
pub const VFS_ROOT_ID: InodeID = 0;
pub const VFS_STDIN_ID: InodeID = 1;
pub const VFS_STDOUT_ID: InodeID = 2;
pub const VFS_STDERR_ID: InodeID = 3;

pub struct DirEntryWrapper {
    /// ino is an inode number
    pub ino: u64,
    /// type is the file type
    pub ty: VfsNodeType,
    /// filename (null-terminated)
    pub name: RRefVec<u8>,
    pub name_len: usize,
}

impl DirEntryWrapper {
    pub fn new(name: RRefVec<u8>) -> Self {
        Self {
            ino: 0,
            ty: VfsNodeType::Unknown,
            name,
            name_len: 0,
        }
    }
}
#[proxy(VfsDomainProxy,Vec<u8>)]
pub trait VfsDomain: Basic + DowncastSync {
    fn init(&self, initrd: &[u8]) -> AlienResult<()>;
    fn vfs_poll(&self, inode: InodeID, events: VfsPollEvents) -> AlienResult<VfsPollEvents>;
    fn vfs_ioctl(&self, inode: InodeID, cmd: u32, arg: usize) -> AlienResult<usize>;
    fn vfs_open(
        &self,
        root: InodeID,
        path: &RRefVec<u8>,
        mode: u32,
        open_flags: usize,
    ) -> AlienResult<InodeID>;
    fn vfs_close(&self, inode: InodeID) -> AlienResult<()>;
    fn vfs_getattr(
        &self,
        inode: InodeID,
        attr: RRef<VfsFileStat>,
    ) -> AlienResult<RRef<VfsFileStat>>;
    fn vfs_read_at(
        &self,
        inode: InodeID,
        offset: u64,
        buf: RRefVec<u8>,
    ) -> AlienResult<(RRefVec<u8>, usize)>;

    fn vfs_read(&self, inode: InodeID, buf: RRefVec<u8>) -> AlienResult<(RRefVec<u8>, usize)>;

    fn vfs_write_at(
        &self,
        inode: InodeID,
        offset: u64,
        buf: RRefVec<u8>,
    ) -> AlienResult<(RRefVec<u8>, usize)>;
    fn vfs_write(&self, inode: InodeID, buf: &RRefVec<u8>) -> AlienResult<usize>;
    fn vfs_flush(&self, inode: InodeID) -> AlienResult<()>;
    fn vfs_fsync(&self, inode: InodeID) -> AlienResult<()>;
    fn vfs_lseek(&self, inode: InodeID, seek: SeekFrom) -> AlienResult<u64>;
    fn vfs_inode_type(&self, inode: InodeID) -> AlienResult<VfsNodeType>;
    fn vfs_readdir(&self, inode: InodeID, buf: RRefVec<u8>) -> AlienResult<(RRefVec<u8>, usize)>;
    fn vfs_get_path(&self, inode: InodeID, buf: RRefVec<u8>) -> AlienResult<(RRefVec<u8>, usize)>;
    fn do_fcntl(&self, inode: InodeID, cmd: usize, args: usize) -> AlienResult<isize>;
    fn do_pipe2(&self, flags: usize) -> AlienResult<(InodeID, InodeID)>;
    /// Create a socket and return the inode id
    fn do_socket(&self, socket_id: SocketID) -> AlienResult<InodeID>;
    /// Get the socket id from inode id
    fn socket_id(&self, inode: InodeID) -> AlienResult<SocketID>;
}

impl_downcast!(sync VfsDomain);
