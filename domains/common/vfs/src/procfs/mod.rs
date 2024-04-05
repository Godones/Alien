use alloc::sync::Arc;
use core::ops::Index;

use vfscore::{dentry::VfsDentry, path::VfsPath};

use crate::tree::FS;

///
/// ```bash
/// |
/// |-- meminfo
/// |-- interrupts
/// |-- mounts
/// |-- filesystems
/// ```
// todo!(use ramfs instead of dynfs)
pub fn init_procfs(root_dt: Arc<dyn VfsDentry>) -> Arc<dyn VfsDentry> {
    let path = VfsPath::new(root_dt.clone(), root_dt.clone());
    let ramfs = FS.lock().index("ramfs").clone();
    let fake_ramfs = ramfs.i_mount(0, "/proc/self", None, &[]).unwrap();
    path.join("self").unwrap().mount(fake_ramfs, 0).unwrap();

    path.join("self/exe")
        .unwrap()
        .symlink("/bin/busybox")
        .unwrap();

    basic::println!("procfs init success");

    root_dt
}
