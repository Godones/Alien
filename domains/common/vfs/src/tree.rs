use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    sync::Arc,
};

use basic::{constants::io::OpenFlags, println, sync::Mutex};
use interface::{DomainType, VFS_ROOT_ID, VFS_STDERR_ID, VFS_STDIN_ID, VFS_STDOUT_ID};
use rref::RRefVec;
use spin::{Lazy, Once};
use vfscore::{dentry::VfsDentry, fstype::VfsFsType, path::VfsPath, VfsResult};

use crate::{
    devfs, insert_dentry, kfile::KernelFile, pipefs, procfs, ramfs::init_ramfs,
    shim::RootShimDentry, sys, VFS_MAP,
};

pub static FS: Lazy<Mutex<BTreeMap<String, Arc<dyn VfsFsType>>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));

static SYSTEM_ROOT_FS: Once<Arc<dyn VfsDentry>> = Once::new();

fn common_load_or_create_fs(
    create: bool,
    name: &str,
    mp: &[u8],
    func: Option<fn(root: &Arc<dyn VfsDentry>)>,
) -> Arc<dyn VfsDentry> {
    let fs_domain = if create {
        basic::create_domain(name).unwrap()
    } else {
        basic::get_domain(name).unwrap()
    };
    let root = match fs_domain {
        DomainType::FsDomain(fs) => {
            let mp = RRefVec::from_slice(mp);
            let root_inode_id = fs.mount(&mp, None).unwrap();
            let shim_root_dentry: Arc<dyn VfsDentry> =
                Arc::new(RootShimDentry::new(fs, root_inode_id));
            match func {
                Some(f) => f(&shim_root_dentry),
                None => {}
            }
            shim_root_dentry
        }
        _ => panic!("{} domain not found", name),
    };
    root
}

/// Init the filesystem
pub fn init_filesystem(initrd: &[u8]) -> VfsResult<()> {
    let ramfs_root = common_load_or_create_fs(false, "ramfs-1", b"/", Some(init_ramfs));
    SYSTEM_ROOT_FS.call_once(|| ramfs_root.clone());

    let procfs_root =
        common_load_or_create_fs(false, "procfs", b"/proc", Some(procfs::init_procfs));

    let devfs_domain = basic::get_domain("devfs").unwrap();
    let devfs_root = match devfs_domain {
        DomainType::DevFsDomain(devfs) => {
            let mp = RRefVec::from_slice(b"/dev");
            let root_inode_id = devfs.mount(&mp, None).unwrap();
            let shim_root_dentry: Arc<dyn VfsDentry> =
                Arc::new(RootShimDentry::new(devfs.clone(), root_inode_id));
            devfs::init_devfs(&devfs, &shim_root_dentry);
            shim_root_dentry
        }
        _ => panic!("devfs domain not found"),
    };

    let sysfs_root = common_load_or_create_fs(false, "sysfs", b"/sys", Some(sys::init_sysfs));
    let tmpfs_root = common_load_or_create_fs(true, "ramfs", b"/tmp", None);
    let _pipefs_root =
        common_load_or_create_fs(false, "pipefs", b"/pipe", Some(pipefs::init_pipefs));
    let shm_ramfs_root = common_load_or_create_fs(true, "ramfs", b"/dev/shm", None);

    let path = VfsPath::new(ramfs_root.clone(), ramfs_root.clone());
    path.join("proc")?.mount(procfs_root, 0)?;
    path.join("sys")?.mount(sysfs_root, 0)?;
    path.join("dev")?.mount(devfs_root, 0)?;
    path.join("tmp")?.mount(tmpfs_root.clone(), 0)?;
    path.join("dev/shm")?.mount(shm_ramfs_root, 0)?;

    crate::initrd::populate_initrd(ramfs_root.clone(), initrd)?;

    {
        let mut map = VFS_MAP.write();
        map.insert(
            VFS_ROOT_ID,
            Arc::new(KernelFile::new(ramfs_root.clone(), OpenFlags::O_RDWR)),
        );
        map.insert(VFS_STDIN_ID, STDIN.clone());
        map.insert(VFS_STDOUT_ID, STDOUT.clone());
        map.insert(VFS_STDERR_ID, STDOUT.clone());
    }

    let fatfs_domain = basic::get_domain("fatfs-1").unwrap();
    match fatfs_domain {
        DomainType::FsDomain(fatfs) => {
            let blk_inode = path
                .join("/dev/sda")?
                .open(None)
                .expect("open /dev/sda failed");
            let id = insert_dentry(blk_inode, OpenFlags::O_RDWR);
            let mp = RRefVec::from_slice(b"/tests");
            let root_inode_id = fatfs.mount(&mp, Some(id)).unwrap();
            let shim_inode = Arc::new(RootShimDentry::new(fatfs, root_inode_id));
            path.join("tests")?.mount(shim_inode, 0)?;
        }
        _ => panic!("fatfs domain not found"),
    }
    println!("Vfs Tree:");
    vfscore::path::print_fs_tree(&mut VfsOutPut, ramfs_root.clone(), "".to_string(), false)
        .unwrap();
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
#[allow(unused)]
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
