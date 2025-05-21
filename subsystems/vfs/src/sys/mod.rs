mod cpu;
mod event_source;

use alloc::sync::Arc;

use dynfs::DynFsDirInode;
use vfscore::{dentry::VfsDentry, error::VfsError, fstype::VfsFsType, inode::VfsInode};

use crate::{sys::cpu::CpuPossible, CommonFsProviderImpl};

pub type SysFsDirInodeImpl = DynFsDirInode<CommonFsProviderImpl, spin::Mutex<()>>;

fn create_sys_dir_in(
    parent: &Arc<SysFsDirInodeImpl>,
    dir_name: &str,
    perm: &str,
) -> Arc<SysFsDirInodeImpl> {
    let dir = parent.add_dir_manually(dir_name, perm.into()).unwrap();
    dir.downcast_arc::<SysFsDirInodeImpl>()
        .map_err(|_| VfsError::Invalid)
        .unwrap()
}

fn create_sys_file_in(
    parent: &Arc<SysFsDirInodeImpl>,
    name: &str,
    file: Arc<dyn VfsInode>,
    perm: &str,
) -> Arc<dyn VfsInode> {
    let file = parent.add_file_manually(name, file, perm.into()).unwrap();
    file
}

pub fn init_sysfs(sysfs: Arc<dyn VfsFsType>) -> Arc<dyn VfsDentry> {
    let root_dt = sysfs.i_mount(0, "/sys", None, &[]).unwrap();
    let root_inode = root_dt.inode().unwrap();
    let root_inode = root_inode
        .downcast_arc::<SysFsDirInodeImpl>()
        .map_err(|_| VfsError::Invalid)
        .unwrap();

    let devices = create_sys_dir_in(&root_inode, "devices", "r-xr-xr-x");
    let system = create_sys_dir_in(&devices, "system", "r-xr-xr-x");
    let cpu = create_sys_dir_in(&system, "cpu", "r-xr-xr-x");
    create_sys_file_in(&cpu, "possible", Arc::new(CpuPossible), "r--r--r--");
    create_sys_file_in(&cpu, "online", Arc::new(CpuPossible), "r--r--r--");

    let bus = create_sys_dir_in(&root_inode, "bus", "r-xr-xr-x");
    let event_source = create_sys_dir_in(&bus, "event_source", "r-xr-xr-x");
    let devices = create_sys_dir_in(&event_source, "devices", "r-xr-xr-x");
    let kprobe = create_sys_dir_in(&devices, "kprobe", "r-xr-xr-x");
    let _kprobe_ty = create_sys_file_in(
        &kprobe,
        "type",
        Arc::new(event_source::KprobeType),
        "r--r--r--",
    );

    println!("sysfs init success");
    root_dt
}
