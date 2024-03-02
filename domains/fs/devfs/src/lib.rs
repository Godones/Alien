#![no_std]
#![deny(unsafe_code)]

mod id;
mod null;
mod random;

extern crate alloc;
extern crate malloc;

use crate::id::{alloc_device_id, DeviceId};
use crate::null::NullDevice;
use crate::random::RandomDevice;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::fmt::Debug;
use devfs::DevKernelProvider;
use interface::{Basic, FsDomain};
use ksync::Mutex;
use log::info;
use spin::Lazy;
use vfscore::dentry::VfsDentry;
use vfscore::fstype::VfsFsType;
use vfscore::inode::VfsInode;
use vfscore::utils::{VfsNodeType, VfsTimeSpec};

#[derive(Clone)]
pub struct DevFsProviderImpl;

impl DevKernelProvider for DevFsProviderImpl {
    fn current_time(&self) -> VfsTimeSpec {
        VfsTimeSpec::new(0, 0)
    }
    fn rdev2device(&self, rdev: u64) -> Option<Arc<dyn VfsInode>> {
        let device_id = DeviceId::from(rdev);
        DEVICES.lock().get(&device_id).cloned()
    }
}

pub static DEVICES: Lazy<Mutex<BTreeMap<DeviceId, Arc<dyn VfsInode>>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));

pub fn register_device(inode: Arc<dyn VfsInode>) {
    let rdev = inode.get_attr().unwrap().st_rdev;
    let device_id = DeviceId::from(rdev);
    DEVICES.lock().insert(device_id, inode);
}

pub fn unregister_device(rdev: DeviceId) {
    DEVICES.lock().remove(&rdev);
}

type DevFs = devfs::DevFs<DevFsProviderImpl, Mutex<()>>;

#[derive(Debug)]
pub struct DevFsDomain {
    fs: Arc<DevFs>,
}

impl DevFsDomain {
    pub fn new() -> Self {
        let fs = Arc::new(DevFs::new(DevFsProviderImpl));
        DevFsDomain { fs }
    }
}

impl Basic for DevFsDomain {}

impl FsDomain for DevFsDomain {}

///```bash
/// |
/// |-- null
/// |-- zero
/// |-- random
/// |-- urandom
/// |-- tty
/// |-- shm (a ramfs will be mounted here)
/// |-- misc
///    |-- rtc
/// ```
pub fn init_devfs(devfs: Arc<dyn VfsFsType>) -> Arc<dyn VfsDentry> {
    let root = devfs.i_mount(0, "/dev", None, &[]).unwrap();
    let root_inode = root.inode().unwrap();

    let null_device = Arc::new(NullDevice::new(alloc_device_id(VfsNodeType::CharDevice)));
    let zero_device = Arc::new(NullDevice::new(alloc_device_id(VfsNodeType::CharDevice)));
    let random_device = Arc::new(RandomDevice::new(alloc_device_id(VfsNodeType::CharDevice)));
    let urandom_device = Arc::new(RandomDevice::new(alloc_device_id(VfsNodeType::CharDevice)));

    root_inode
        .create(
            "null",
            'c'.into(),
            "rw-rw-rw-".into(),
            Some(null_device.device_id().id()),
        )
        .unwrap();
    root_inode
        .create(
            "zero",
            'c'.into(),
            "rw-rw-rw-".into(),
            Some(zero_device.device_id().id()),
        )
        .unwrap();
    root_inode
        .create(
            "random",
            'c'.into(),
            "rw-rw-rw-".into(),
            Some(random_device.device_id().id()),
        )
        .unwrap();
    root_inode
        .create(
            "urandom",
            'c'.into(),
            "rw-rw-rw-".into(),
            Some(urandom_device.device_id().id()),
        )
        .unwrap();

    register_device(null_device);
    register_device(zero_device);
    register_device(random_device);
    register_device(urandom_device);

    root_inode
        .create("shm", VfsNodeType::Dir, "rwxrwxrwx".into(), None)
        .unwrap();
    root_inode
        .create("misc", VfsNodeType::Dir, "rwxrwxrwx".into(), None)
        .unwrap();

    scan_system_devices(root_inode);
    // todo!(tty,shm,misc)
    libsyscall::println!("devfs init success");
    root
}

fn scan_system_devices(root: Arc<dyn VfsInode>) {
    BLOCK_DEVICE.get().map(|blk| {
        let block_device = Arc::new(BLKDevice::new(
            alloc_device_id(VfsNodeType::BlockDevice),
            blk.clone(),
        ));
        root.create(
            "sda",
            VfsNodeType::BlockDevice,
            "rw-rw----".into(),
            Some(block_device.device_id().id()),
        )
        .unwrap();
        info!("block device id: {}", block_device.device_id().id());
        register_device(block_device);
    });
    GPU_DEVICE.get().map(|gpu| {
        let gpu_device = Arc::new(GPUDevice::new(
            alloc_device_id(VfsNodeType::CharDevice),
            gpu.clone(),
        ));
        root.create(
            "gpu",
            VfsNodeType::BlockDevice,
            "rw-rw----".into(),
            Some(gpu_device.device_id().id()),
        )
        .unwrap();
        info!("gpu device id: {}", gpu_device.device_id().id());
        register_device(gpu_device);
    });
    KEYBOARD_INPUT_DEVICE.get().map(|input| {
        let input_device = Arc::new(INPUTDevice::new(
            alloc_device_id(VfsNodeType::CharDevice),
            input.clone(),
            false,
        ));
        root.create(
            "keyboard",
            VfsNodeType::BlockDevice,
            "rw-rw----".into(),
            Some(input_device.device_id().id()),
        )
        .unwrap();
        info!("keyboard device id: {}", input_device.device_id().id());
        register_device(input_device);
    });
    MOUSE_INPUT_DEVICE.get().map(|input| {
        let input_device = Arc::new(INPUTDevice::new(
            alloc_device_id(VfsNodeType::CharDevice),
            input.clone(),
            true,
        ));
        root.create(
            "mouse",
            VfsNodeType::BlockDevice,
            "rw-rw----".into(),
            Some(input_device.device_id().id()),
        )
        .unwrap();
        info!("mouse device id: {}", input_device.device_id().id());
        register_device(input_device);
    });
    RTC_DEVICE.get().map(|rtc| {
        let rtc_device = Arc::new(RTCDevice::new(
            alloc_device_id(VfsNodeType::CharDevice),
            rtc.clone(),
        ));
        root.create(
            "rtc",
            VfsNodeType::BlockDevice,
            "rw-rw----".into(),
            Some(rtc_device.device_id().id()),
        )
        .unwrap();
        info!("rtc device id: {}", rtc_device.device_id().id());
        register_device(rtc_device);
    });
    UART_DEVICE.get().map(|uart| {
        let uart_device = Arc::new(UARTDevice::new(
            alloc_device_id(VfsNodeType::CharDevice),
            uart.clone(),
        ));
        root.create(
            "tty",
            VfsNodeType::BlockDevice,
            "rw-rw----".into(),
            Some(uart_device.device_id().id()),
        )
        .unwrap();
        info!("uart device id: {}", uart_device.device_id().id());
        register_device(uart_device);
    });
}

pub fn main() -> Arc<dyn FsDomain> {
    Arc::new(DevFsDomain::new())
}
