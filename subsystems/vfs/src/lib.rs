#![feature(c_variadic)]
#![no_std]

extern crate alloc;
#[macro_use]
extern crate platform;
use crate::dev::{DevFsProviderImpl};
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use constants::{AlienResult};
use core::ops::Index;
use dynfs::DynFsKernelProvider;
use ksync::Mutex;
use spin::{Lazy, Once};
use vfscore::dentry::VfsDentry;
use vfscore::fstype::VfsFsType;
use vfscore::path::VfsPath;
use vfscore::utils::VfsTimeSpec;
#[cfg(feature = "ext")]
use vfscore::inode::VfsInode;
#[cfg(feature = "ext")]
mod extffi;
pub mod dev;
pub mod kfile;
pub mod pipefs;
pub mod proc;
pub mod ram;
pub mod sys;

pub static FS: Lazy<Mutex<BTreeMap<String, Arc<dyn VfsFsType>>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));

static SYSTEM_ROOT_FS: Once<Arc<dyn VfsDentry>> = Once::new();

type SysFs = dynfs::DynFs<CommonFsProviderImpl, Mutex<()>>;
type ProcFs = dynfs::DynFs<CommonFsProviderImpl, Mutex<()>>;
type RamFs = ramfs::RamFs<CommonFsProviderImpl, Mutex<()>>;
type DevFs = devfs::DevFs<DevFsProviderImpl, Mutex<()>>;
type TmpFs = ramfs::RamFs<CommonFsProviderImpl, Mutex<()>>;
type PipeFs = dynfs::DynFs<CommonFsProviderImpl, Mutex<()>>;

#[cfg(feature = "fat")]
type DiskFs = fat_vfs::FatFs<CommonFsProviderImpl, Mutex<()>>;

#[cfg(feature = "ext")]
type DiskFs = lwext4_vfs::ExtFs<CommonFsProviderImpl, Mutex<()>>;

#[derive(Clone)]
pub struct CommonFsProviderImpl;

impl DynFsKernelProvider for CommonFsProviderImpl {
    fn current_time(&self) -> VfsTimeSpec {
        VfsTimeSpec::new(0, 0)
    }
}

impl ramfs::RamFsProvider for CommonFsProviderImpl {
    fn current_time(&self) -> VfsTimeSpec {
        DynFsKernelProvider::current_time(self)
    }
}

#[cfg(feature = "fat")]
impl fat_vfs::FatFsProvider for CommonFsProviderImpl {
    fn current_time(&self) -> VfsTimeSpec {
        DynFsKernelProvider::current_time(self)
    }
}

#[cfg(feature = "ext")]
impl lwext4_vfs::ExtDevProvider for CommonFsProviderImpl {
    fn rdev2device(&self, rdev: u64) -> Option<Arc<dyn VfsInode>> {
        use dev::{DEVICES};
        use constants::DeviceId;
        let device_id = DeviceId::from(rdev);
        DEVICES.lock().get(&device_id).cloned()
    }
}

fn register_all_fs() {
    let procfs = Arc::new(ProcFs::new(CommonFsProviderImpl, "procfs"));
    let sysfs = Arc::new(SysFs::new(CommonFsProviderImpl, "sysfs"));
    let ramfs = Arc::new(RamFs::new(CommonFsProviderImpl));
    let devfs = Arc::new(DevFs::new(DevFsProviderImpl));
    let tmpfs = Arc::new(TmpFs::new(CommonFsProviderImpl));
    let pipefs = Arc::new(PipeFs::new(CommonFsProviderImpl, "pipefs"));

    FS.lock().insert("procfs".to_string(), procfs);
    FS.lock().insert("sysfs".to_string(), sysfs);
    FS.lock().insert("ramfs".to_string(), ramfs);
    FS.lock().insert("devfs".to_string(), devfs);
    FS.lock().insert("tmpfs".to_string(), tmpfs);
    FS.lock().insert("pipefs".to_string(), pipefs);

    #[cfg(feature = "fat")]
        let diskfs = Arc::new(DiskFs::new(CommonFsProviderImpl));
    #[cfg(feature = "ext")]
        let diskfs = Arc::new(DiskFs::new(lwext4_vfs::ExtFsType::Ext4,CommonFsProviderImpl));

    FS.lock().insert("diskfs".to_string(), diskfs);

    println!("register fs success");
}

/// Init the filesystem
pub fn init_filesystem() -> AlienResult<()> {
    register_all_fs();
    let ramfs_root = ram::init_ramfs(FS.lock().index("ramfs").clone());
    let procfs_root = proc::init_procfs(FS.lock().index("procfs").clone());
    let devfs_root = dev::init_devfs(FS.lock().index("devfs").clone());
    let sysfs_root = sys::init_sysfs(FS.lock().index("sysfs").clone());
    let tmpfs_root = FS
        .lock()
        .index("tmpfs")
        .clone()
        .i_mount(0, "/tmp", None, &[])?;

    pipefs::init_pipefs(FS.lock().index("pipefs").clone());

    let path = VfsPath::new(ramfs_root.clone());
    path.join("proc")?.mount(procfs_root, 0)?;
    path.join("sys")?.mount(sysfs_root, 0)?;
    path.join("dev")?.mount(devfs_root, 0)?;
    path.join("tmp")?.mount(tmpfs_root.clone(), 0)?;

    let shm_ramfs = FS
        .lock()
        .index("ramfs")
        .clone()
        .i_mount(0, "/dev/shm", None, &[])?;
    path.join("dev/shm")?.mount(shm_ramfs, 0)?;

    let diskfs = FS.lock().index("diskfs").clone();
    let blk_inode = path
        .join("/dev/sda")?
        .open(None)
        .expect("open /dev/sda failed")
        .inode()?;
    let diskfs_root = diskfs.i_mount(0, "/bin", Some(blk_inode), &[])?;
    path.join("bin")?.mount(diskfs_root, 0)?;

    vfscore::path::print_fs_tree(&mut VfsOutPut, ramfs_root.clone(), "".to_string(), false)
        .unwrap();
    SYSTEM_ROOT_FS.call_once(|| ramfs_root);
    println!("Init filesystem success");
    Ok(())
}

struct VfsOutPut;
impl core::fmt::Write for VfsOutPut {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        platform::console::console_write(s);
        Ok(())
    }
}

/// Get the root filesystem of the system
#[inline]
pub fn system_root_fs() -> Arc<dyn VfsDentry> {
    SYSTEM_ROOT_FS.get().unwrap().clone()
}

/// Get the filesystem by name
#[inline]
pub fn system_support_fs(fs_name: &str) -> Option<Arc<dyn VfsFsType>> {
    FS.lock().iter().find_map(|(name, fs)| {
        if name == fs_name {
            Some(fs.clone())
        } else {
            None
        }
    })
}
