mod block;
mod gpu;
mod id;
mod null;
mod random;
mod rtc;
mod uart;

use crate::devfs::block::BLKDevice;
use crate::devfs::gpu::GPUDevice;
use crate::devfs::rtc::RTCDevice;
use crate::devfs::uart::UARTDevice;
use alloc::sync::Arc;
use constants::DeviceId;
use devfs::DevKernelProvider;
use id::{alloc_device_id, query_device, register_device};
use log::info;
use null::NullDevice;
use random::RandomDevice;
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
        query_device(device_id)
    }
}

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
    let uart = libsyscall::get_uart_domain();
    let gpu = libsyscall::get_gpu_domain();
    // let mouse = libsyscall::get_input_domain("mouse").unwrap();
    // let keyboard = libsyscall::get_input_domain("keyboard").unwrap();
    let blk = libsyscall::get_cache_blk_domain();
    let rtc = libsyscall::get_rtc_domain();

    blk.map(|blk| {
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
    gpu.map(|gpu| {
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
    // KEYBOARD_INPUT_DEVICE.get().map(|input| {
    //     let input_device = Arc::new(INPUTDevice::new(
    //         alloc_device_id(VfsNodeType::CharDevice),
    //         input.clone(),
    //         false,
    //     ));
    //     root.create(
    //         "keyboard",
    //         VfsNodeType::BlockDevice,
    //         "rw-rw----".into(),
    //         Some(input_device.device_id().id()),
    //     )
    //     .unwrap();
    //     info!("keyboard device id: {}", input_device.device_id().id());
    //     register_device(input_device);
    // });
    // MOUSE_INPUT_DEVICE.get().map(|input| {
    //     let input_device = Arc::new(INPUTDevice::new(
    //         alloc_device_id(VfsNodeType::CharDevice),
    //         input.clone(),
    //         true,
    //     ));
    //     root.create(
    //         "mouse",
    //         VfsNodeType::BlockDevice,
    //         "rw-rw----".into(),
    //         Some(input_device.device_id().id()),
    //     )
    //     .unwrap();
    //     info!("mouse device id: {}", input_device.device_id().id());
    //     register_device(input_device);
    // });
    rtc.map(|rtc| {
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
    uart.map(|uart| {
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
