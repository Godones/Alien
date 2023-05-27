use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

use fat32_vfs::fstype::FAT;
use lazy_static::lazy_static;
use rvfs::dentry::DirEntry;
use rvfs::file::{vfs_open_file, vfs_read_file, FileMode, OpenFlags};
use rvfs::info::{ProcessFs, ProcessFsInfo, VfsTime};
use rvfs::mount::{do_mount, MountFlags, VfsMount};
use rvfs::mount_rootfs;
use rvfs::superblock::{register_filesystem, DataOps, Device};
use spin::Mutex;

use crate::driver::rtc::get_rtc_time;
use crate::driver::QEMU_BLOCK_DEVICE;
use crate::task::current_process;

// only call once before the first process is created
lazy_static! {
    pub static ref TMP_MNT: Mutex<Arc<VfsMount>> = Mutex::new(Arc::new(VfsMount::empty()));
    pub static ref TMP_DIR: Mutex<Arc<DirEntry>> = Mutex::new(Arc::new(DirEntry::empty()));
}

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
    println!("vfs init done");
}

pub fn read_all(file_name: &str, buf: &mut Vec<u8>) -> bool {
    let file = vfs_open_file::<VfsProvider>(file_name, OpenFlags::O_RDONLY, FileMode::FMODE_READ);
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
        let mut tmp = vec![0; 512 as usize];
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
        if let Some(process) = current_process() {
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
