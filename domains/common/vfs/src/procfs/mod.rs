use alloc::sync::Arc;

use basic::println;
use interface::DomainType;
use rref::RRefVec;
use vfscore::{dentry::VfsDentry, path::VfsPath};

use crate::shim::RootShimDentry;

///
/// ```bash
/// |
/// |-- meminfo
/// |-- interrupts
/// |-- mounts
/// |-- filesystems
/// ```
// todo!(use ramfs instead of dynfs)
pub fn init_procfs(root_dt: &Arc<dyn VfsDentry>) {
    let path = VfsPath::new(root_dt.clone(), root_dt.clone());
    let ramfs_domain = basic::create_domain("ramfs").unwrap();
    let ramfs_root = match ramfs_domain {
        DomainType::FsDomain(ramfs) => {
            let mp = RRefVec::from_slice(b"/proc/self");
            let root_inode_id = ramfs.mount(&mp, None).unwrap();
            let shim_root_dentry = Arc::new(RootShimDentry::new(ramfs, root_inode_id));
            shim_root_dentry
        }
        _ => panic!("ramfs domain not create"),
    };
    path.join("self").unwrap().mount(ramfs_root, 0).unwrap();
    path.join("self/exe")
        .unwrap()
        .symlink("/bin/busybox")
        .unwrap();
    println!("procfs init success");
}
