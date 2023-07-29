use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::ptr::null;

use fat32_vfs::fstype::FAT;
use lazy_static::lazy_static;
use rvfs::dentry::DirEntry;
use rvfs::devfs::DEVFS_TYPE;
use rvfs::file::{
    FileMode, OpenFlags, vfs_mkdir, vfs_mknod, vfs_open_file, vfs_read_file, vfs_write_file,
};
use rvfs::info::{ProcessFs, ProcessFsInfo, VfsTime};
use rvfs::inode::{InodeMode, SpecialData};
use rvfs::mount::{do_mount, MountFlags, VfsMount};
use rvfs::mount_rootfs;
use rvfs::ramfs::tmpfs::TMP_FS_TYPE;
use rvfs::superblock::{DataOps, Device, register_filesystem};

use kernel_sync::Mutex;

use crate::config::{MEMINFO, PASSWORD, RTC_TIME, UTC};
use crate::driver::QEMU_BLOCK_DEVICE;
use crate::driver::rtc::get_rtc_time;
use crate::task::current_task;

// only call once before the first process is created
lazy_static! {
    pub static ref TMP_MNT: Mutex<Arc<VfsMount>> = Mutex::new(Arc::new(VfsMount::empty()));
    pub static ref TMP_DIR: Mutex<Arc<DirEntry>> = Mutex::new(Arc::new(DirEntry::empty()));
}

const MOUNT_INFO: &str = r"
 rootfs / rootfs rw 0 0
 devfs /dev devfs rw 0 0
 fat32 / fat rw 0 0
";

/// after call this function, user should set the fs info for the first process
pub fn init_vfs() {
    // init the rootfs
    let root_mnt = mount_rootfs();
    *TMP_MNT.lock() = root_mnt.clone();
    *TMP_DIR.lock() = root_mnt.root.clone();
    // mount fat fs
    register_filesystem(FAT).expect("register fat fs failed");
    let img_device = QEMU_BLOCK_DEVICE.lock()[0].clone();
    let data = Box::new(Fat32Data::new(img_device));
    let mnt = do_mount::<VfsProvider>("fat", "/", "fat", MountFlags::empty(), Some(data)).unwrap();
    *TMP_MNT.lock() = mnt.clone();
    *TMP_DIR.lock() = mnt.root.clone();
    vfs_mkdir::<VfsProvider>("/dev", FileMode::FMODE_RDWR).unwrap();
    vfs_mkdir::<VfsProvider>("/tmp", FileMode::FMODE_RDWR).unwrap();

    register_filesystem(DEVFS_TYPE).unwrap();
    do_mount::<VfsProvider>("none", "/dev", "devfs", MountFlags::MNT_NO_DEV, None).unwrap();
    #[cfg(any(feature = "vf2", feature = "sifive"))]
    do_mount::<VfsProvider>("root", "/tmp", "rootfs", MountFlags::MNT_NO_DEV, None).unwrap();
    vfs_mknod::<VfsProvider>(
        "/dev/null",
        InodeMode::S_CHARDEV,
        FileMode::FMODE_RDWR,
        u32::MAX,
    )
        .unwrap();
    vfs_mknod::<VfsProvider>("/dev/zero", InodeMode::S_CHARDEV, FileMode::FMODE_RDWR, 0).unwrap();

    register_filesystem(TMP_FS_TYPE).unwrap();
    vfs_mkdir::<VfsProvider>("/dev/shm", FileMode::FMODE_RDWR).unwrap();
    do_mount::<VfsProvider>("none", "/dev/shm", "tmpfs", MountFlags::MNT_NO_DEV, None).unwrap();

    prepare_root();
    prepare_proc();
    prepare_etc();
    prepare_test_need();
    prepare_dev();
    prepare_var();

    // let fake_sort_src = vfs_open_file::<VfsProvider>("/sort.src", OpenFlags::O_CREAT | OpenFlags::O_RDWR, FileMode::FMODE_RDWR).unwrap();
    // vfs_write_file::<VfsProvider>(fake_sort_src.clone(), SORT_SRC, 0).unwrap();
    // vfs_close_file::<VfsProvider>(fake_sort_src).unwrap();
    println!("vfs init success");
}

fn prepare_var() {
    vfs_mkdir::<VfsProvider>("/var", FileMode::FMODE_RDWR).unwrap();
    vfs_mkdir::<VfsProvider>("/var/log", FileMode::FMODE_RDWR).unwrap();
    vfs_mkdir::<VfsProvider>("/var/tmp", FileMode::FMODE_RDWR).unwrap();
    #[cfg(any(feature = "vf2", feature = "sifive"))]
    do_mount::<VfsProvider>("none", "/var/tmp", "tmpfs", MountFlags::MNT_NO_DEV, None).unwrap();
}

fn prepare_root() {
    vfs_mkdir::<VfsProvider>("/root", FileMode::FMODE_RDWR).unwrap();
    let _bash_profile = vfs_open_file::<VfsProvider>(
        "/root/.bashrc",
        OpenFlags::O_RDWR | OpenFlags::O_CREAT,
        FileMode::FMODE_RDWR,
    )
        .unwrap();
}

fn prepare_dev() {
    vfs_mkdir::<VfsProvider>("/dev/misc", FileMode::FMODE_RDWR).unwrap();
    do_mount::<VfsProvider>("none", "/dev/misc", "tmpfs", MountFlags::MNT_NO_DEV, None).unwrap();
    let rtc_file = vfs_open_file::<VfsProvider>(
        "/dev/misc/rtc",
        OpenFlags::O_RDWR | OpenFlags::O_CREAT,
        FileMode::FMODE_RDWR,
    )
        .unwrap();

    rtc_file
        .f_dentry
        .access_inner()
        .d_inode
        .access_inner()
        .special_data = Some(SpecialData::CharData(null()));
    rtc_file.access_inner().f_ops_ext.ioctl = |_, _, _| 0;
    vfs_write_file::<VfsProvider>(rtc_file, RTC_TIME.as_bytes(), 0).unwrap();
    vfs_mknod::<VfsProvider>("/dev/tty", InodeMode::S_CHARDEV, FileMode::FMODE_RDWR, 0).unwrap();
}

fn prepare_test_need() {
    vfs_open_file::<VfsProvider>(
        "/test.txt",
        OpenFlags::O_RDWR | OpenFlags::O_CREAT,
        FileMode::FMODE_RDWR,
    )
        .unwrap();
}

fn prepare_proc() {
    vfs_mkdir::<VfsProvider>("/proc", FileMode::FMODE_RDWR).unwrap();
    do_mount::<VfsProvider>("none", "/proc", "tmpfs", MountFlags::MNT_NO_DEV, None).unwrap();
    let file = vfs_open_file::<VfsProvider>(
        "/proc/mounts",
        OpenFlags::O_RDWR | OpenFlags::O_CREAT,
        FileMode::FMODE_RDWR,
    )
        .unwrap();
    vfs_write_file::<VfsProvider>(file, MOUNT_INFO.as_bytes(), 0).unwrap();
    let mem_info = vfs_open_file::<VfsProvider>(
        "/proc/meminfo",
        OpenFlags::O_RDWR | OpenFlags::O_CREAT,
        FileMode::FMODE_RDWR,
    )
        .unwrap();
    vfs_write_file::<VfsProvider>(mem_info, MEMINFO.as_bytes(), 0).unwrap();
}

fn prepare_etc() {
    vfs_mkdir::<VfsProvider>("/etc", FileMode::FMODE_RDWR).unwrap();
    do_mount::<VfsProvider>("none", "/etc", "tmpfs", MountFlags::MNT_NO_DEV, None).unwrap();
    let file = vfs_open_file::<VfsProvider>(
        "/etc/localtime",
        OpenFlags::O_RDWR | OpenFlags::O_CREAT,
        FileMode::FMODE_RDWR,
    )
        .unwrap();
    vfs_write_file::<VfsProvider>(file, UTC, 0).unwrap();
    let adjtime_file = vfs_open_file::<VfsProvider>(
        "/etc/adjtime",
        OpenFlags::O_RDWR | OpenFlags::O_CREAT,
        FileMode::FMODE_RDWR,
    )
        .unwrap();
    vfs_write_file::<VfsProvider>(adjtime_file, RTC_TIME.as_bytes(), 0).unwrap();

    let password = vfs_open_file::<VfsProvider>(
        "/etc/passwd",
        OpenFlags::O_RDWR | OpenFlags::O_CREAT,
        FileMode::FMODE_RDWR,
    )
        .unwrap();
    vfs_write_file::<VfsProvider>(password, PASSWORD.as_bytes(), 0).unwrap();
}

pub fn read_all(file_name: &str, buf: &mut Vec<u8>) -> bool {
    let file = vfs_open_file::<VfsProvider>(file_name, OpenFlags::O_RDONLY, FileMode::FMODE_RDWR);
    if file.is_err() {
        warn!("open file {} failed", file_name);
        return false;
    }
    let file = file.unwrap();
    let size = file
        .f_dentry
        .access_inner()
        .d_inode
        .access_inner()
        .file_size;
    let mut offset = 0;
    while offset < size {
        let mut tmp = vec![0; 512usize];
        let res = vfs_read_file::<VfsProvider>(file.clone(), &mut tmp, offset as u64).unwrap();
        offset += res;
        buf.extend_from_slice(&tmp);
    }
    assert_eq!(offset, size);
    true
}

#[derive(Debug)]
pub struct Fat32Data {
    device: Arc<dyn Device>,
}

impl Fat32Data {
    pub fn new(device: Arc<dyn Device>) -> Self {
        Fat32Data { device }
    }
}

impl DataOps for Fat32Data {
    fn device(&self, _: &str) -> Option<Arc<dyn Device>> {
        Some(self.device.clone())
    }
}

pub struct VfsProvider;

impl ProcessFs for VfsProvider {
    fn get_fs_info() -> ProcessFsInfo {
        if let Some(process) = current_task() {
            let inner = process.access_inner();
            inner.fs_info.clone().into()
        } else {
            let mnt = TMP_MNT.lock().clone();
            let dir = TMP_DIR.lock().clone();
            ProcessFsInfo::new(mnt.clone(), dir.clone(), dir, mnt)
        }
    }
    // 检查进程的链接文件嵌套查询深度是否超过最大值
    fn check_nested_link() -> bool {
        false
    }
    // 更新进程链接文件的相关数据： 嵌套查询深度/ 调用链接查找的次数
    fn update_link_data() {}
    fn max_link_count() -> u32 {
        10
    }
    fn current_time() -> VfsTime {
        get_rtc_time().map_or(VfsTime::default(), |x| VfsTime {
            year: x.year,
            month: x.month,
            day: x.day,
            hour: x.hour,
            minute: x.minute,
            second: x.second,
        })
    }
}

// static SORT_SRC: &[u8] = include_bytes!("../../../sdcard/sort.src");
