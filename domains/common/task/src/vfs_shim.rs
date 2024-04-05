use alloc::{sync::Arc, vec::Vec};

use constants::{io::OpenFlags, AlienError, AlienResult, AT_FDCWD};
use interface::{InodeID, VfsDomain, VFS_ROOT_ID, VFS_STDIN_ID, VFS_STDOUT_ID};
use rref::{RRef, RRefVec};
use spin::{Lazy, Once};
use vfscore::utils::VfsFileStat;

use crate::processor::current_task;

static VFS_DOMAIN: Once<Arc<dyn VfsDomain>> = Once::new();

pub fn init_vfs_domain(vfs_domain: Arc<dyn VfsDomain>) {
    VFS_DOMAIN.call_once(|| vfs_domain);
}

pub static STDIN: Lazy<Arc<ShimFile>> = Lazy::new(|| Arc::new(ShimFile::new(VFS_STDIN_ID)));

pub static STDOUT: Lazy<Arc<ShimFile>> = Lazy::new(|| Arc::new(ShimFile::new(VFS_STDOUT_ID)));

/// equal to Arc<dyn VfsDentry>
#[derive(Debug)]
pub struct ShimFile {
    id: InodeID,
}

impl ShimFile {
    pub const fn new(id: InodeID) -> Self {
        Self { id }
    }
    pub fn inode_id(&self) -> InodeID {
        self.id
    }

    fn get_attr(&self) -> AlienResult<RRef<VfsFileStat>> {
        let attr = RRef::new(VfsFileStat::default());
        let res = VFS_DOMAIN.get().unwrap().vfs_getattr(self.id, attr);
        res
    }

    fn read_at(&self, offset: u64, buf: RRefVec<u8>) -> AlienResult<(RRefVec<u8>, usize)> {
        let res = VFS_DOMAIN.get().unwrap().vfs_read_at(self.id, offset, buf);
        res
    }
}

pub fn read_all(file_name: &str, buf: &mut Vec<u8>) -> bool {
    let task = current_task();
    let path = if task.is_none() {
        (VFS_ROOT_ID, VFS_ROOT_ID)
    } else {
        user_path_at(AT_FDCWD, file_name).unwrap()
    };

    let name = RRefVec::from_slice(file_name.as_bytes());
    let res = VFS_DOMAIN
        .get()
        .unwrap()
        .vfs_open(path.0, &name, 0, OpenFlags::O_RDONLY.bits());

    if res.is_err() {
        info!("open file {} failed, err:{:?}", file_name, res.err());
        return false;
    }
    let shim_file = ShimFile::new(res.unwrap());

    let size = shim_file.get_attr().unwrap().st_size;
    let mut offset = 0;
    let mut tmp = RRefVec::new(0, 512);
    let mut res;
    while offset < size {
        (tmp, res) = shim_file.read_at(offset, tmp).unwrap();
        offset += res as u64;
        buf.extend_from_slice(&tmp.as_slice()[..res]);
    }
    assert_eq!(offset, size);
    true
}

fn user_path_at(fd: isize, path: &str) -> AlienResult<(InodeID, InodeID)> {
    info!("user_path_at fd: {},path:{}", fd, path);
    let task = current_task().unwrap();
    let res = if !path.starts_with("/") {
        if fd == AT_FDCWD {
            let fs_context = &task.inner().fs_info;
            (VFS_ROOT_ID, fs_context.cwd)
        } else {
            let fd = fd as usize;
            let file = task.get_file(fd).ok_or(AlienError::EBADF)?;
            (VFS_ROOT_ID, file.inode_id())
        }
    } else {
        (VFS_ROOT_ID, VFS_ROOT_ID)
    };
    Ok(res)
}
