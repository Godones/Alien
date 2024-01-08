use crate::config::AT_FDCWD;
use crate::fs::dev::{init_devfs, DevFsProviderImpl};
use crate::fs::proc::init_procfs;
use crate::fs::ram::init_ramfs;
use crate::fs::sys::init_sysfs;
use crate::ipc::init_pipefs;
use crate::print::console::Stdout;
use crate::task::{current_task, FsContext};
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
pub use basic::*;
use constants::io::InodeMode;
use constants::AlienResult;
use constants::LinuxErrno;
pub use control::*;
use core::ops::Index;
use dynfs::DynFsKernelProvider;
use fat_vfs::FatFsProvider;
use ksync::Mutex;
pub use poll::*;
use riscv::register::sstatus::FS;
pub use select::*;
use spin::{Lazy, Once};
pub use stdio::*;
use vfscore::dentry::VfsDentry;
use vfscore::fstype::VfsFsType;
use vfscore::path::{SysContext, VfsPath};
use vfscore::utils::{VfsInodeMode, VfsNodeType, VfsTimeSpec};

pub mod stdio;

pub mod basic;
pub mod control;
pub mod dev;
pub mod ext;
pub mod file;
pub mod link;
pub mod poll;
pub mod proc;
pub mod ram;
pub mod select;
pub mod sys;

pub static FS: Lazy<Mutex<BTreeMap<String, Arc<dyn VfsFsType>>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));

pub static SYSTEM_ROOT_FS: Once<Arc<dyn VfsDentry>> = Once::new();

type SysFs = dynfs::DynFs<CommonFsProviderImpl, Mutex<()>>;
type ProcFs = dynfs::DynFs<CommonFsProviderImpl, Mutex<()>>;
type RamFs = ramfs::RamFs<CommonFsProviderImpl, Mutex<()>>;
type DevFs = devfs::DevFs<DevFsProviderImpl, Mutex<()>>;
type TmpFs = ramfs::RamFs<CommonFsProviderImpl, Mutex<()>>;
type PipeFs = dynfs::DynFs<CommonFsProviderImpl, Mutex<()>>;
type FatFs = fat_vfs::FatFs<CommonFsProviderImpl, Mutex<()>>;

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

impl FatFsProvider for CommonFsProviderImpl {
    fn current_time(&self) -> VfsTimeSpec {
        DynFsKernelProvider::current_time(self)
    }
}

fn register_all_fs() {
    let procfs = Arc::new(ProcFs::new(CommonFsProviderImpl, "procfs"));
    let sysfs = Arc::new(SysFs::new(CommonFsProviderImpl, "sysfs"));
    let ramfs = Arc::new(RamFs::new(CommonFsProviderImpl));
    let devfs = Arc::new(DevFs::new(DevFsProviderImpl));
    let tmpfs = Arc::new(TmpFs::new(CommonFsProviderImpl));
    let pipefs = Arc::new(PipeFs::new(CommonFsProviderImpl, "pipefs"));

    let fatfs = Arc::new(FatFs::new(CommonFsProviderImpl));

    FS.lock().insert("procfs".to_string(), procfs);
    FS.lock().insert("sysfs".to_string(), sysfs);
    FS.lock().insert("ramfs".to_string(), ramfs);
    FS.lock().insert("devfs".to_string(), devfs);
    FS.lock().insert("tmpfs".to_string(), tmpfs);
    FS.lock().insert("pipefs".to_string(), pipefs);
    FS.lock().insert("fatfs".to_string(), fatfs);
    println!("register fs success");
}

/// Init the filesystem
pub fn init_filesystem() -> AlienResult<()> {
    register_all_fs();
    let ramfs_root = init_ramfs(FS.lock().index("ramfs").clone());
    let procfs_root = init_procfs(FS.lock().index("procfs").clone());
    let devfs_root = init_devfs(FS.lock().index("devfs").clone());
    let sysfs_root = init_sysfs(FS.lock().index("sysfs").clone());
    let tmpfs_root = FS
        .lock()
        .index("tmpfs")
        .clone()
        .i_mount(0, "/tmp", None, &[])?;

    init_pipefs(FS.lock().index("pipefs").clone());

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

    let fatfs = FS.lock().index("fatfs").clone();
    let blk_inode = path
        .join("/dev/sda")?
        .open(None)
        .expect("open /dev/sda failed")
        .inode()?;
    let fat32_root = fatfs.i_mount(0, "/bin", Some(blk_inode), &[])?;
    path.join("bin")?.mount(fat32_root, 0)?;

    vfscore::path::print_fs_tree(&mut VfsOutPut, ramfs_root.clone(), "".to_string(), false)
        .unwrap();
    SYSTEM_ROOT_FS.call_once(|| ramfs_root);
    println!("Init filesystem success");
    Ok(())
}

struct VfsOutPut;
impl core::fmt::Write for VfsOutPut {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        Stdout.write_str(s).unwrap();
        Ok(())
    }
}

/// 地址解析函数，通过 `fd` 所指向的一个目录文件 和 相对于该目录文件的路径或绝对路径 `path` 解析出某目标文件的绝对路径。
///
/// 当传入的`path`是一个相对地址时，那么`path`会被解析成基于文件描述符`fd`所指向的目录地址的一个地址；当传入的`path`是一个相对地址并且
/// `fd`被特殊的设置为`AT_FDCWD`时，`path`会被解析成基于调用该系统调用的进程当前工作目录的一个地址；当传入的`path`是一个绝对地址时，`fd`将被直接忽略。
///
/// 在`Alien`使用的`rvfs`中，对一个文件路径`path`是相对路径还是绝对路径的的判断条件如下：
/// + 绝对路径：以`/`开头，如`/file1.txt`，表示根目录下的`file1.txt`文件；
/// + 相对路径: 以`./`或者`../`或者其它开头，如`./file1.txt`，表示`dirfd`所指向的目录下的`file1.txt`文件。
fn user_path_at(fd: isize, path: &str) -> AlienResult<VfsPath> {
    info!("user_path_at fd: {},path:{}", fd, path);
    let process = current_task().unwrap();
    let res = if !path.starts_with("/") {
        if fd == AT_FDCWD {
            let fs_context = process.access_inner().fs_info.clone();
            VfsPath::new(fs_context.cwd).join(path)
        } else {
            let fd = fd as usize;
            let file = process.get_file(fd).ok_or(LinuxErrno::EBADF)?;
            VfsPath::new(file.dentry()).join(path)
        }
    } else {
        VfsPath::new(SYSTEM_ROOT_FS.get().unwrap().clone()).join(path)
    };
    res.map_err(|e| e.into())
}

pub fn read_all(file_name: &str, buf: &mut Vec<u8>) -> bool {
    let task = current_task();
    // let cwd = if task.is_some() {
    //     task.unwrap().access_inner().cwd().cwd
    // } else {
    //     SYSTEM_ROOT_FS.get().unwrap().clone()
    // };
    let path = if task.is_none() {
        VfsPath::new(SYSTEM_ROOT_FS.get().unwrap().clone())
            .join(file_name)
            .unwrap()
    } else {
        user_path_at(AT_FDCWD, file_name).unwrap()
    };

    let dentry = path.open(None);
    if dentry.is_err() {
        info!("open file {} failed, err:{:?}", file_name, dentry.err());
        return false;
    }
    let dentry = dentry.unwrap();
    if dentry.inode().unwrap().inode_type() != VfsNodeType::File {
        info!("{} is not a file", file_name);
        return false;
    }
    let size = dentry.inode().unwrap().get_attr().unwrap().st_size;
    let mut offset = 0;
    while offset < size {
        let mut tmp = vec![0; 512usize];
        let res = dentry.inode().unwrap().read_at(offset, &mut tmp).unwrap();
        offset += res as u64;
        buf.extend_from_slice(&tmp);
    }
    assert_eq!(offset, size);
    true
}

/// [InodeMode](InodeMode)转换为[VfsInodeMode](VfsInodeMode)
fn im2vim(mode: InodeMode) -> VfsInodeMode {
    VfsInodeMode::from_bits_truncate(mode.bits())
}

fn syscontext_for_vfs(fs_info: FsContext) -> SysContext {
    SysContext {
        pid: 0,
        uid: 0,
        gid: 0,
        cwd: fs_info.cwd.clone(),
        root: fs_info.root.clone(),
    }
}
