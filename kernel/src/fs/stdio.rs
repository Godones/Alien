use alloc::sync::Arc;
use constants::io::OpenFlags;
use spin::Lazy;
use vfs::kfile::KernelFile;
use vfs::system_root_fs;
use vfscore::path::VfsPath;

type Stdin = KernelFile;
type Stdout = KernelFile;

pub static STDIN: Lazy<Arc<Stdin>> = Lazy::new(|| {
    let path = VfsPath::new(system_root_fs(), system_root_fs())
        .join("dev/tty")
        .unwrap();
    let dentry = path.open(None).unwrap();
    let file = KernelFile::new(dentry, OpenFlags::O_RDONLY);
    Arc::new(file)
});

pub static STDOUT: Lazy<Arc<Stdout>> = Lazy::new(|| {
    let path = VfsPath::new(system_root_fs(), system_root_fs())
        .join("dev/tty")
        .unwrap();
    let dentry = path.open(None).unwrap();
    let file = KernelFile::new(dentry, OpenFlags::O_WRONLY);
    Arc::new(file)
});
