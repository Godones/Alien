use crate::fs::file::KernelFile;
use crate::fs::SYSTEM_ROOT_FS;
use alloc::sync::Arc;
use pconst::io::OpenFlags;
use spin::Lazy;
use vfscore::path::VfsPath;

type Stdin = KernelFile;
type Stdout = KernelFile;

pub static STDIN: Lazy<Arc<Stdin>> = Lazy::new(|| {
    let path = VfsPath::new(SYSTEM_ROOT_FS.get().unwrap().clone())
        .join("dev/tty")
        .unwrap();
    let dentry = path.open(None).unwrap();
    let file = KernelFile::new(dentry, OpenFlags::O_RDONLY);
    Arc::new(file)
});

pub static STDOUT: Lazy<Arc<Stdout>> = Lazy::new(|| {
    let path = VfsPath::new(SYSTEM_ROOT_FS.get().unwrap().clone())
        .join("dev/tty")
        .unwrap();
    let dentry = path.open(None).unwrap();
    let file = KernelFile::new(dentry, OpenFlags::O_WRONLY);
    Arc::new(file)
});
