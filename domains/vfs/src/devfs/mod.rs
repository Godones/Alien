mod block;
mod gpu;
mod id;
mod input;
mod null;
mod random;
mod rtc;
mod uart;

use crate::devfs::block::BLKDevice;
use crate::devfs::gpu::GPUDevice;
use crate::devfs::input::INPUTDevice;
use crate::devfs::rtc::RTCDevice;
use crate::devfs::uart::UARTDevice;
use alloc::sync::Arc;
use basic::println;
use constants::DeviceId;
use devfs::DevKernelProvider;
use id::{alloc_device_id, query_device, register_device};
use interface::DomainType;
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
    println!("devfs init success");
    root
}

fn scan_system_devices(root: Arc<dyn VfsInode>) {
    let uart = basic::get_domain("uart");
    let gpu = basic::get_domain("gpu");
    let mouse = basic::get_domain("mouse");
    let keyboard = basic::get_domain("keyboard");
    let blk = basic::get_domain("cache_blk");
    let rtc = basic::get_domain("rtc");

    match uart {
        Some(DomainType::UartDomain(uart)) => {
            let uart_device = Arc::new(UARTDevice::new(
                alloc_device_id(VfsNodeType::CharDevice),
                uart,
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
        }
        _ => {
            panic!("uart domain not found");
        }
    };

    match gpu {
        Some(DomainType::GpuDomain(gpu)) => {
            let gpu_device = Arc::new(GPUDevice::new(
                alloc_device_id(VfsNodeType::CharDevice),
                gpu,
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
        }
        _ => {
            println!("gpu domain not found");
        }
    };

    match mouse {
        Some(DomainType::InputDomain(input)) => {
            let input_device = Arc::new(INPUTDevice::new(
                alloc_device_id(VfsNodeType::CharDevice),
                input,
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
        }
        _ => {
            println!("mouse domain not found");
        }
    };
    match keyboard {
        Some(DomainType::InputDomain(input)) => {
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
        }
        _ => {
            println!("keyboard domain not found");
        }
    };

    match blk {
        Some(DomainType::CacheBlkDeviceDomain(blk)) => {
            let block_device = Arc::new(BLKDevice::new(
                alloc_device_id(VfsNodeType::BlockDevice),
                blk,
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
        }
        _ => panic!("blk domain not found"),
    };

    match rtc {
        Some(DomainType::RtcDomain(rtc)) => {
            let rtc_device = Arc::new(RTCDevice::new(
                alloc_device_id(VfsNodeType::CharDevice),
                rtc,
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
        }
        _ => {
            println!("rtc domain not found");
        }
    };
}
