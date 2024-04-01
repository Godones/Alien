mod id;

use alloc::sync::Arc;

use basic::println;
use id::alloc_device_id;
use interface::DevFsDomain;
use rref::RRefVec;
use vfscore::{dentry::VfsDentry, utils::VfsNodeType};

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
pub fn init_devfs(devfs_domain: &Arc<dyn DevFsDomain>, root_dt: &Arc<dyn VfsDentry>) {
    let root_inode = root_dt.inode().unwrap();

    let null_device_id = alloc_device_id(VfsNodeType::CharDevice);
    let zero_device_id = alloc_device_id(VfsNodeType::CharDevice);
    let random_device_id = alloc_device_id(VfsNodeType::CharDevice);
    let urandom_device_id = alloc_device_id(VfsNodeType::CharDevice);

    devfs_domain
        .register(null_device_id.id(), &RRefVec::from_slice(b"null"))
        .unwrap();
    devfs_domain
        .register(zero_device_id.id(), &RRefVec::from_slice(b"zero"))
        .unwrap();
    devfs_domain
        .register(random_device_id.id(), &RRefVec::from_slice(b"random"))
        .unwrap();
    devfs_domain
        .register(urandom_device_id.id(), &RRefVec::from_slice(b"urandom"))
        .unwrap();
    root_inode
        .create(
            "null",
            'c'.into(),
            "rw-rw-rw-".into(),
            Some(null_device_id.id()),
        )
        .unwrap();
    root_inode
        .create(
            "zero",
            'c'.into(),
            "rw-rw-rw-".into(),
            Some(zero_device_id.id()),
        )
        .unwrap();
    root_inode
        .create(
            "random",
            'c'.into(),
            "rw-rw-rw-".into(),
            Some(random_device_id.id()),
        )
        .unwrap();
    root_inode
        .create(
            "urandom",
            'c'.into(),
            "rw-rw-rw-".into(),
            Some(urandom_device_id.id()),
        )
        .unwrap();

    root_inode
        .create("shm", VfsNodeType::Dir, "rwxrwxrwx".into(), None)
        .unwrap();
    root_inode
        .create("misc", VfsNodeType::Dir, "rwxrwxrwx".into(), None)
        .unwrap();

    scan_system_devices(devfs_domain, root_dt);

    // todo!(tty,shm,misc)
    println!("devfs init success");
}

pub fn scan_system_devices(devfs_domain: &Arc<dyn DevFsDomain>, root_dt: &Arc<dyn VfsDentry>) {
    let root = root_dt.inode().unwrap();

    let uart = basic::get_domain("buf_uart");
    let gpu = basic::get_domain("gpu");
    let mouse = basic::get_domain("mouse");
    let keyboard = basic::get_domain("keyboard");
    let blk = basic::get_domain("cache_blk");
    let rtc = basic::get_domain("rtc");

    match uart {
        Some(_) => {
            let uart_id = alloc_device_id(VfsNodeType::CharDevice);
            devfs_domain
                .register(uart_id.id(), &RRefVec::from_slice(b"buf_uart"))
                .unwrap();
            root.create(
                "tty",
                VfsNodeType::CharDevice,
                "rw-rw----".into(),
                Some(uart_id.id()),
            )
            .unwrap();
        }
        None => {
            panic!("uart domain not found");
        }
    }

    match gpu {
        Some(_) => {
            let gpu_id = alloc_device_id(VfsNodeType::CharDevice);
            devfs_domain
                .register(gpu_id.id(), &RRefVec::from_slice(b"virtio-mmio-gpu"))
                .unwrap();
            root.create(
                "virtio-mmio-gpu",
                VfsNodeType::CharDevice,
                "rw-rw----".into(),
                Some(gpu_id.id()),
            )
            .unwrap();
        }
        None => {
            println!("gpu domain not found");
        }
    }

    match mouse {
        Some(_) => {
            let mouse_id = alloc_device_id(VfsNodeType::CharDevice);
            devfs_domain
                .register(mouse_id.id(), &RRefVec::from_slice(b"mouse"))
                .unwrap();
            root.create(
                "mouse",
                VfsNodeType::CharDevice,
                "rw-rw----".into(),
                Some(mouse_id.id()),
            )
            .unwrap();
        }
        None => {
            println!("mouse domain not found");
        }
    }

    match keyboard {
        Some(_) => {
            let keyboard_id = alloc_device_id(VfsNodeType::CharDevice);
            devfs_domain
                .register(keyboard_id.id(), &RRefVec::from_slice(b"keyboard"))
                .unwrap();
            root.create(
                "keyboard",
                VfsNodeType::CharDevice,
                "rw-rw----".into(),
                Some(keyboard_id.id()),
            )
            .unwrap();
        }
        None => {
            println!("keyboard domain not found");
        }
    }

    match blk {
        Some(_) => {
            let blk_id = alloc_device_id(VfsNodeType::BlockDevice);
            devfs_domain
                .register(blk_id.id(), &RRefVec::from_slice(b"cache_blk"))
                .unwrap();
            root.create(
                "sda",
                VfsNodeType::BlockDevice,
                "rw-rw----".into(),
                Some(blk_id.id()),
            )
            .unwrap();
        }
        None => panic!("blk domain not found"),
    }

    match rtc {
        Some(_) => {
            let rtc_id = alloc_device_id(VfsNodeType::CharDevice);
            devfs_domain
                .register(rtc_id.id(), &RRefVec::from_slice(b"rtc"))
                .unwrap();
            root.create(
                "rtc",
                VfsNodeType::CharDevice,
                "rw-rw----".into(),
                Some(rtc_id.id()),
            )
            .unwrap();
        }
        None => {
            println!("rtc domain not found");
        }
    };
    root.create("shm", VfsNodeType::Dir, "rwxrwxrwx".into(), None)
        .unwrap();
    root.create("misc", VfsNodeType::Dir, "rwxrwxrwx".into(), None)
        .unwrap();
}
