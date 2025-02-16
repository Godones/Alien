use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    sync::Weak,
    vec::Vec,
};

use fatfs::{Error, Seek};
use vfscore::{
    error::VfsError,
    file::VfsFile,
    impl_dir_inode_default,
    inode::{InodeAttr, VfsInode},
    superblock::VfsSuperBlock,
    utils::{
        VfsDirEntry, VfsFileStat, VfsInodeMode, VfsNodePerm, VfsNodeType, VfsRenameFlag, VfsTime,
    },
    DVec, VfsResult,
};

use crate::{
    fs::FatFsSuperBlock,
    inode::{FatFsFileInode, FatFsInodeSame},
    *,
};

pub struct FatFsDirInode<R: VfsRawMutex> {
    #[allow(unused)]
    parent: Weak<Mutex<R, FatDir>>,
    dir: Arc<Mutex<R, FatDir>>,
    attr: FatFsInodeSame<R>,
    inode_cache: Mutex<R, BTreeMap<String, Arc<dyn VfsInode>>>,
}

impl<R: VfsRawMutex + 'static> FatFsDirInode<R> {
    pub fn new(
        parent: &Arc<Mutex<R, FatDir>>,
        dir: Arc<Mutex<R, FatDir>>,
        sb: &Arc<FatFsSuperBlock<R>>,
        perm: VfsNodePerm,
    ) -> Self {
        Self {
            parent: Arc::downgrade(parent),
            dir,
            attr: FatFsInodeSame::new(sb, perm),
            inode_cache: Mutex::new(BTreeMap::new()),
        }
    }

    fn delete_file(&self, name: &str, ty: VfsNodeType) -> VfsResult<()> {
        let mut inode_cache = self.inode_cache.lock();
        let dir = self.dir.lock();
        let file = if let Some((_, inode)) = inode_cache.iter().find(|(k, _)| *k == name) {
            assert_eq!(inode.inode_type(), ty);
            let inode = inode_cache.remove(name).unwrap();
            let r = inode
                .downcast_arc::<FatFsFileInode<R>>()
                .map_err(|_| VfsError::Invalid)?;
            Some(r.raw_file())
        } else {
            None
        };
        if ty == VfsNodeType::File {
            let action = |file: &mut FatFile| -> VfsResult<()> {
                file.seek(fatfs::SeekFrom::Start(0))
                    .map_err(|_| VfsError::IoError)?;
                file.truncate().map_err(|_| VfsError::IoError)?;
                Ok(())
            };
            match file {
                Some(f) => action(&mut f.lock()),
                None => {
                    let mut file = dir.open_file(name).map_err(|e| match e {
                        Error::NotFound | Error::InvalidInput => VfsError::NoEntry,
                        _ => VfsError::IoError,
                    })?;
                    action(&mut file)
                }
            }?;
        }
        if ty == VfsNodeType::Dir {
            let _dir = dir.open_dir(name).map_err(|e| match e {
                Error::NotFound | Error::InvalidInput => VfsError::NoEntry,
                _ => VfsError::IoError,
            })?;
        }
        dir.remove(name).map_err(|e| match e {
            Error::NotFound | Error::InvalidInput => VfsError::NoEntry,
            _ => VfsError::IoError,
        })?;
        Ok(())
    }
}

impl<R: VfsRawMutex + 'static> VfsFile for FatFsDirInode<R> {
    fn readdir(&self, start_index: usize) -> VfsResult<Option<VfsDirEntry>> {
        let entry = self.dir.lock().iter().nth(start_index);
        if let Some(entry) = entry {
            match entry {
                Ok(entry) => {
                    let ty = if entry.is_dir() {
                        VfsNodeType::Dir
                    } else {
                        VfsNodeType::File
                    };
                    let entry = VfsDirEntry {
                        ino: 1,
                        ty,
                        name: entry.file_name(),
                    };
                    Ok(Some(entry))
                }
                Err(_e) => Err(VfsError::IoError),
            }
        } else {
            Ok(None)
        }
    }
    fn ioctl(&self, _cmd: u32, _arg: usize) -> VfsResult<usize> {
        Err(VfsError::NoTTY)
    }
}

impl<R: VfsRawMutex + 'static> VfsInode for FatFsDirInode<R> {
    fn get_super_block(&self) -> VfsResult<Arc<dyn VfsSuperBlock>> {
        let sb = self.attr.sb.upgrade().unwrap();
        Ok(sb)
    }

    fn node_perm(&self) -> VfsNodePerm {
        self.attr.inner.lock().perm
    }

    fn create(
        &self,
        name: &str,
        ty: VfsNodeType,
        perm: VfsNodePerm,
        _rdev: Option<u64>,
    ) -> VfsResult<Arc<dyn VfsInode>> {
        let mut inode_cache = self.inode_cache.lock();
        if inode_cache.contains_key(name) {
            return Err(VfsError::EExist);
        }
        match ty {
            VfsNodeType::Dir => {
                let new_dir = self
                    .dir
                    .lock()
                    .create_dir(name)
                    .map_err(|_| VfsError::IoError)?;
                let new_dir = Arc::new(Mutex::new(new_dir));

                let inode =
                    FatFsDirInode::new(&self.dir, new_dir, &self.attr.sb.upgrade().unwrap(), perm);
                let inode = Arc::new(inode);
                inode_cache.insert(name.to_string(), inode.clone());
                Ok(inode)
            }
            VfsNodeType::File => {
                let file = self
                    .dir
                    .lock()
                    .create_file(name)
                    .map_err(|_| VfsError::IoError)?;
                let file = Arc::new(Mutex::new(file));
                let inode = FatFsFileInode::new(
                    &self.dir,
                    file,
                    &self.attr.sb.upgrade().unwrap(),
                    name.to_string(),
                    perm,
                );
                let inode = Arc::new(inode);
                inode_cache.insert(name.to_string(), inode.clone());
                Ok(inode)
            }
            _ => Err(VfsError::Invalid),
        }
    }

    fn link(&self, _name: &str, _src: Arc<dyn VfsInode>) -> VfsResult<Arc<dyn VfsInode>> {
        Err(VfsError::NoSys)
    }

    fn unlink(&self, name: &str) -> VfsResult<()> {
        self.delete_file(name, VfsNodeType::File)
    }

    fn symlink(&self, _name: &str, _sy_name: &str) -> VfsResult<Arc<dyn VfsInode>> {
        Err(VfsError::NoSys)
    }
    fn lookup(&self, name: &str) -> VfsResult<Arc<dyn VfsInode>> {
        let mut inode_cache = self.inode_cache.lock();
        if let Some(inode) = inode_cache.get(name) {
            return Ok(inode.clone());
        }
        let dir = self.dir.lock();

        let find = dir
            .iter()
            .find(|e| {
                let entry = e.as_ref().unwrap();
                let e_name = entry.file_name();
                name == e_name
            })
            .ok_or(VfsError::NoEntry)?;
        let entry = find.map_err(|_| VfsError::IoError)?;

        if entry.is_dir() {
            let new_dir = dir
                .open_dir(name)
                .map_err(|e| !matches!(e, Error::NotFound | Error::InvalidInput));
            if new_dir.is_ok() {
                let new_dir = new_dir.unwrap();
                let new_dir = Arc::new(Mutex::new(new_dir));
                let inode = FatFsDirInode::new(
                    &self.dir,
                    new_dir,
                    &self.attr.sb.upgrade().unwrap(),
                    VfsNodePerm::default_dir(),
                );
                let inode = Arc::new(inode);
                inode_cache.insert(name.to_string(), inode.clone());
                return Ok(inode);
            }
            Err(VfsError::IoError)
        } else {
            let file = dir.open_file(name).map_err(|e| match e {
                Error::NotFound | Error::InvalidInput => VfsError::NoEntry,
                _ => VfsError::IoError,
            })?;
            let file = Arc::new(Mutex::new(file));
            drop(dir);
            let inode = FatFsFileInode::new(
                &self.dir,
                file,
                &self.attr.sb.upgrade().unwrap(),
                name.to_string(),
                VfsNodePerm::default_file(),
            );
            let inode = Arc::new(inode);
            inode_cache.insert(name.to_string(), inode.clone());
            Ok(inode)
        }
    }

    fn rmdir(&self, name: &str) -> VfsResult<()> {
        self.delete_file(name, VfsNodeType::Dir)
    }

    fn set_attr(&self, _attr: InodeAttr) -> VfsResult<()> {
        Ok(())
    }

    fn get_attr(&self) -> VfsResult<VfsFileStat> {
        let attr = self.attr.inner.lock();
        let mode = VfsInodeMode::from(attr.perm, VfsNodeType::Dir).bits();
        Ok(VfsFileStat {
            st_dev: 0,
            st_ino: 1,
            st_mode: mode,
            st_nlink: 1,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            __pad: 0,
            st_size: 4096,
            st_blksize: 512,
            __pad2: 0,
            st_blocks: 0,
            st_atime: attr.atime,
            st_mtime: attr.mtime,
            st_ctime: attr.ctime,
            unused: 0,
        })
    }

    impl_dir_inode_default!();

    fn list_xattr(&self) -> VfsResult<Vec<String>> {
        Err(VfsError::NoSys)
    }

    fn inode_type(&self) -> VfsNodeType {
        VfsNodeType::Dir
    }

    fn rename_to(
        &self,
        old_name: &str,
        new_parent: Arc<dyn VfsInode>,
        new_name: &str,
        flag: VfsRenameFlag,
    ) -> VfsResult<()> {
        let dir = self.dir.lock();
        if flag.contains(VfsRenameFlag::RENAME_EXCHANGE) {
            return Err(VfsError::NoSys);
        } else {
            let new_parent = new_parent
                .downcast_arc::<FatFsDirInode<R>>()
                .map_err(|_| VfsError::Invalid)?;
            if Arc::ptr_eq(&self.dir, &new_parent.dir) {
                let _ = dir.remove(new_name);
                dir.rename(old_name, &*dir, new_name).map_err(|e| match e {
                    Error::NotFound => VfsError::NoEntry,
                    Error::AlreadyExists => VfsError::EExist,
                    _ => VfsError::IoError,
                })?;
            } else {
                let _ = dir.remove(new_name);
                dir.rename(old_name, &*new_parent.dir.lock(), new_name)
                    .map_err(|e| match e {
                        Error::NotFound => VfsError::NoEntry,
                        Error::AlreadyExists => VfsError::EExist,
                        _ => VfsError::IoError,
                    })?;
            };
            self.inode_cache.lock().remove(old_name);
            new_parent.inode_cache.lock().remove(new_name);
        }
        Ok(())
    }
    fn update_time(&self, time: VfsTime, now: VfsTimeSpec) -> VfsResult<()> {
        let mut attr = self.attr.inner.lock();
        match time {
            VfsTime::AccessTime(t) => attr.atime = t,
            VfsTime::ModifiedTime(t) => attr.mtime = t,
        }
        attr.ctime = now;
        Ok(())
    }
}
