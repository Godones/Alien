mod dir;
mod file;

use alloc::sync::Weak;

pub use dir::*;
pub use file::*;
use vfscore::utils::VfsNodePerm;

use crate::{fs::FatFsSuperBlock, *};

struct FatFsInodeSame<R: VfsRawMutex> {
    pub sb: Weak<FatFsSuperBlock<R>>,
    pub inner: Mutex<R, FatFsInodeAttr>,
}
struct FatFsInodeAttr {
    pub atime: VfsTimeSpec,
    pub mtime: VfsTimeSpec,
    pub ctime: VfsTimeSpec,
    pub perm: VfsNodePerm,
}

impl<R: VfsRawMutex> FatFsInodeSame<R> {
    pub fn new(sb: &Arc<FatFsSuperBlock<R>>, perm: VfsNodePerm) -> Self {
        Self {
            sb: Arc::downgrade(sb),
            inner: Mutex::new(FatFsInodeAttr {
                atime: VfsTimeSpec::new(0, 0),
                mtime: VfsTimeSpec::new(0, 0),
                ctime: VfsTimeSpec::new(0, 0),
                perm,
            }),
        }
    }
}
