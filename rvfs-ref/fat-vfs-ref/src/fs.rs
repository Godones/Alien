use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    sync::Weak,
};

use fatfs::FileSystem;
use log::info;
use unifs::dentry::UniFsDentry;
use vfscore::{
    dentry::VfsDentry,
    error::VfsError,
    fstype::{FileSystemFlags, VfsFsType},
    inode::VfsInode,
    superblock::{SuperType, VfsSuperBlock},
    utils::{VfsFsStat, VfsNodeType},
    VfsResult,
};

use super::*;
use crate::{device::FatDevice, inode::FatFsDirInode};

pub struct FatFs<T: Send + Sync, R: VfsRawMutex> {
    #[allow(unused)]
    provider: T,
    fs_container: Mutex<R, BTreeMap<usize, Arc<FatFsSuperBlock<R>>>>,
}

impl<T: Send + Sync, R: VfsRawMutex> FatFs<T, R> {
    pub fn new(provider: T) -> Self {
        Self {
            provider,
            fs_container: Mutex::new(BTreeMap::new()),
        }
    }
}

impl<T: FatFsProvider + 'static, R: VfsRawMutex + 'static> VfsFsType for FatFs<T, R> {
    fn mount(
        self: Arc<Self>,
        _flags: u32,
        ab_mnt: &str,
        dev: Option<Arc<dyn VfsInode>>,
        _data: &[u8],
    ) -> VfsResult<Arc<dyn VfsDentry>> {
        let dev = dev.ok_or(VfsError::Invalid)?;
        if dev.inode_type() != VfsNodeType::BlockDevice {
            return Err(VfsError::Invalid);
        }
        let dev_ino = dev.get_attr()?.st_rdev;
        // For same device, we only mount once, but we will return different dentry according to ab_mnt(absolute mount point)
        if let Some(sb) = self.fs_container.lock().get(&(dev_ino as usize)) {
            return sb.root_dentry(ab_mnt);
        }
        let fat_dev = FatDevice::new(dev);
        let sb = FatFsSuperBlock::<R>::new(&(self.clone() as Arc<dyn VfsFsType>), fat_dev, ab_mnt);
        // we use dev_ino as the key to store the superblock
        self.fs_container
            .lock()
            .insert(dev_ino as usize, sb.clone());
        sb.root_dentry(ab_mnt)
    }

    fn kill_sb(&self, sb: Arc<dyn VfsSuperBlock>) -> VfsResult<()> {
        if let Ok(sb) = sb.downcast_arc::<FatFsSuperBlock<R>>() {
            let dev_ino = sb.fat_dev.device_file.get_attr()?.st_rdev;
            let sb = self.fs_container.lock().remove(&(dev_ino as usize));
            if let Some(sb) = sb {
                // todo!(call unmount)
                sb.fat_dev.device_file.flush()?;
                sb.fat_dev.device_file.fsync()?;
                info!("fatfs: kill_sb: remove sb for dev {}", dev_ino);
                Ok(())
            } else {
                Err(VfsError::Invalid)
            }
        } else {
            Err(VfsError::Invalid)
        }
    }

    fn fs_flag(&self) -> FileSystemFlags {
        FileSystemFlags::REQUIRES_DEV
    }

    fn fs_name(&self) -> String {
        "fatfs".to_string()
    }
}

pub struct FatFsSuperBlock<R: VfsRawMutex> {
    fat_dev: FatDevice,
    fs_type: Weak<dyn VfsFsType>,
    root: Mutex<R, Option<Arc<dyn VfsInode>>>,
    fs: FileSystem<FatDevice, DefaultTimeProvider, LossyOemCpConverter>,
    mnt_info: Mutex<R, BTreeMap<String, Arc<dyn VfsDentry>>>,
}

impl<R: VfsRawMutex + 'static> FatFsSuperBlock<R> {
    pub fn new(fs_type: &Arc<dyn VfsFsType>, device: FatDevice, ab_mnt: &str) -> Arc<Self> {
        let fs = FileSystem::new(device.clone(), fatfs::FsOptions::new()).unwrap();
        let root_disk_dir = Arc::new(Mutex::new(fs.root_dir()));
        let sb = Arc::new(Self {
            fat_dev: device,
            fs_type: Arc::downgrade(fs_type),
            root: Mutex::new(None),
            fs,
            mnt_info: Mutex::new(BTreeMap::new()),
        });
        let root_inode = Arc::new(FatFsDirInode::new(
            &root_disk_dir.clone(),
            root_disk_dir,
            &sb,
            "rwxrwxrwx".into(),
        ));
        sb.root.lock().replace(root_inode.clone());
        let parent = Weak::<UniFsDentry<R>>::new();
        let root_dt = Arc::new(UniFsDentry::<R>::root(root_inode, parent));
        sb.mnt_info.lock().insert(ab_mnt.into(), root_dt.clone());
        sb
    }

    pub fn root_dentry(&self, ab_mnt: &str) -> VfsResult<Arc<dyn VfsDentry>> {
        self.mnt_info.lock().get(ab_mnt).map_or_else(
            || {
                let parent = Weak::<UniFsDentry<R>>::new();
                let inode = self.root.lock().clone().unwrap();
                let new = Arc::new(UniFsDentry::<R>::root(inode, parent));
                self.mnt_info.lock().insert(ab_mnt.into(), new.clone());
                Ok(new as Arc<dyn VfsDentry>)
            },
            |x| Ok(x.clone()),
        )
    }
}

impl<R: VfsRawMutex + 'static> VfsSuperBlock for FatFsSuperBlock<R> {
    fn sync_fs(&self, _wait: bool) -> VfsResult<()> {
        self.fat_dev.device_file.flush()?;
        self.fat_dev.device_file.fsync()?;
        Ok(())
    }

    fn stat_fs(&self) -> VfsResult<VfsFsStat> {
        let stat_fs = self.fs.stats().map_err(|_| VfsError::IoError)?;
        let ft = self.fs.fat_type();
        let f_type = match ft {
            fatfs::FatType::Fat12 => 0x01,
            fatfs::FatType::Fat16 => 0x04,
            fatfs::FatType::Fat32 => 0x0c,
        };
        Ok(VfsFsStat {
            f_type,
            f_bsize: stat_fs.cluster_size() as i64,
            f_blocks: stat_fs.total_clusters() as u64,
            f_bfree: stat_fs.free_clusters() as u64,
            f_bavail: stat_fs.free_clusters() as u64,
            f_files: 0,
            f_ffree: 0,
            f_fsid: [0, 0],
            f_namelen: 255,
            f_frsize: 0,
            f_flags: 0,
            f_spare: [0; 4],
        })
    }

    fn super_type(&self) -> SuperType {
        SuperType::BlockDev
    }

    fn fs_type(&self) -> Arc<dyn VfsFsType> {
        self.fs_type.upgrade().unwrap()
    }
    fn root_inode(&self) -> VfsResult<Arc<dyn VfsInode>> {
        let root = self.root.lock().clone().unwrap();
        Ok(root)
    }
}
