use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    sync::Arc,
};
use core::ops::Index;

use basic::println;
use constants::io::OpenFlags;
use dynfs::DynFsKernelProvider;
use interface::{VFS_ROOT_ID, VFS_STDERR_ID, VFS_STDIN_ID, VFS_STDOUT_ID};
use ksync::Mutex;
use spin::{Lazy, Once};
use vfscore::{dentry::VfsDentry, fstype::VfsFsType, path::VfsPath, utils::VfsTimeSpec, VfsResult};

use crate::{devfs::DevFsProviderImpl, kfile::KernelFile, pipefs, procfs, sys, VFS_MAP};

type SysFs = dynfs::DynFs<CommonFsProviderImpl, Mutex<()>>;
type ProcFs = dynfs::DynFs<CommonFsProviderImpl, Mutex<()>>;
type TmpFs = ::ramfs::RamFs<CommonFsProviderImpl, Mutex<()>>;
type RamFs = ::ramfs::RamFs<CommonFsProviderImpl, Mutex<()>>;
type DevFs = ::devfs::DevFs<DevFsProviderImpl, Mutex<()>>;
type PipeFs = dynfs::DynFs<CommonFsProviderImpl, Mutex<()>>;
type DiskFs = fat_vfs::FatFs<CommonFsProviderImpl, Mutex<()>>;
#[derive(Clone)]
pub struct CommonFsProviderImpl;

impl DynFsKernelProvider for CommonFsProviderImpl {
    fn current_time(&self) -> VfsTimeSpec {
        VfsTimeSpec::new(0, 0)
    }
}

impl ::ramfs::RamFsProvider for CommonFsProviderImpl {
    fn current_time(&self) -> VfsTimeSpec {
        DynFsKernelProvider::current_time(self)
    }
}

impl fat_vfs::FatFsProvider for CommonFsProviderImpl {
    fn current_time(&self) -> VfsTimeSpec {
        DynFsKernelProvider::current_time(self)
    }
}

pub static FS: Lazy<Mutex<BTreeMap<String, Arc<dyn VfsFsType>>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));

static SYSTEM_ROOT_FS: Once<Arc<dyn VfsDentry>> = Once::new();

fn register_all_fs() {
    let procfs = Arc::new(ProcFs::new(CommonFsProviderImpl, "procfs"));
    let sysfs = Arc::new(SysFs::new(CommonFsProviderImpl, "sysfs"));
    let ramfs = Arc::new(RamFs::new(CommonFsProviderImpl));
    let devfs = Arc::new(DevFs::new(DevFsProviderImpl));
    let tmpfs = Arc::new(TmpFs::new(CommonFsProviderImpl));
    let pipefs = Arc::new(PipeFs::new(CommonFsProviderImpl, "pipefs"));
    let diskfs = Arc::new(DiskFs::new(CommonFsProviderImpl));

    FS.lock().insert("procfs".to_string(), procfs);
    FS.lock().insert("sysfs".to_string(), sysfs);
    FS.lock().insert("ramfs".to_string(), ramfs);
    FS.lock().insert("devfs".to_string(), devfs);
    FS.lock().insert("tmpfs".to_string(), tmpfs);
    FS.lock().insert("pipefs".to_string(), pipefs);
    FS.lock().insert("diskfs".to_string(), diskfs);

    println!("register fs success");
}

/// Init the filesystem
pub fn init_filesystem() -> VfsResult<()> {
    register_all_fs();
    let ramfs_root = crate::ramfs::init_ramfs(FS.lock().index("ramfs").clone());
    let procfs = FS.lock().index("procfs").clone();
    let procfs_root = procfs::init_procfs(procfs);
    let devfs_root = crate::devfs::init_devfs(FS.lock().index("devfs").clone());
    let sysfs_root = sys::init_sysfs(FS.lock().index("sysfs").clone());
    let tmpfs_root = FS
        .lock()
        .index("tmpfs")
        .clone()
        .i_mount(0, "/tmp", None, &[])?;

    pipefs::init_pipefs(FS.lock().index("pipefs").clone());

    let path = VfsPath::new(ramfs_root.clone(), ramfs_root.clone());
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

    let diskfs_root = diskfs.i_mount(0, "/tests", Some(blk_inode), &[])?;

    path.join("tests")?.mount(diskfs_root, 0)?;
    println!("Vfs Tree:");
    vfscore::path::print_fs_tree(&mut VfsOutPut, ramfs_root.clone(), "".to_string(), true).unwrap();

    // initrd::populate_initrd(ramfs_root.clone())?;

    SYSTEM_ROOT_FS.call_once(|| ramfs_root.clone());
    let mut map = VFS_MAP.write();
    map.insert(
        VFS_ROOT_ID,
        Arc::new(KernelFile::new(ramfs_root, OpenFlags::O_RDWR)),
    );
    map.insert(VFS_STDIN_ID, STDIN.clone());
    map.insert(VFS_STDOUT_ID, STDOUT.clone());
    map.insert(VFS_STDERR_ID, STDOUT.clone());

    println!("Init filesystem success");
    Ok(())
}

struct VfsOutPut;
impl core::fmt::Write for VfsOutPut {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        basic::write_console(s);
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
