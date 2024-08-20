use alloc::{string::String, sync::Weak, vec, vec::Vec};

use fatfs::{Read, Seek, Write};
use vfscore::{
    error::VfsError,
    file::VfsFile,
    impl_file_inode_default,
    inode::{InodeAttr, VfsInode},
    superblock::VfsSuperBlock,
    utils::{VfsFileStat, VfsInodeMode, VfsNodePerm, VfsNodeType, VfsRenameFlag, VfsTime},
    RRefVec, VfsResult,
};

use crate::{fs::FatFsSuperBlock, inode::FatFsInodeSame, *};

pub struct FatFsFileInode<R: VfsRawMutex> {
    #[allow(unused)]
    parent: Weak<Mutex<R, FatDir>>,
    file: Arc<Mutex<R, FatFile>>,
    attr: FatFsInodeSame<R>,
    #[allow(unused)]
    name: String,
    size: Mutex<R, u64>,
}

impl<R: VfsRawMutex + 'static> FatFsFileInode<R>
where
    R: VfsRawMutex,
{
    pub fn new(
        parent: &Arc<Mutex<R, FatDir>>,
        file: Arc<Mutex<R, FatFile>>,
        sb: &Arc<FatFsSuperBlock<R>>,
        name: String,
        perm: VfsNodePerm,
    ) -> Self {
        let size = parent
            .lock()
            .iter()
            .find(|x| {
                x.as_ref()
                    .is_ok_and(|x| x.is_file() && x.file_name() == name)
            })
            .map(|e| e.unwrap().len())
            .unwrap_or(0);
        Self {
            name,
            parent: Arc::downgrade(parent),
            file,
            attr: FatFsInodeSame::new(sb, perm),
            size: Mutex::new(size),
        }
    }
    pub fn raw_file(&self) -> Arc<Mutex<R, FatFile>> {
        self.file.clone()
    }
}

impl<R: VfsRawMutex + 'static> VfsFile for FatFsFileInode<R> {
    fn read_at(&self, offset: u64, mut o_buf: RRefVec<u8>) -> VfsResult<(RRefVec<u8>, usize)> {
        let mut file = self.file.lock();
        let fat_offset = file.offset();
        if offset != fat_offset as u64 {
            file.seek(fatfs::SeekFrom::Start(offset))
                .map_err(|_| VfsError::IoError)?;
        }
        let mut buf = o_buf.as_mut_slice();
        let mut count = 0;
        while !buf.is_empty() {
            let len = file.read(buf).map_err(|_| VfsError::IoError)?;
            if len == 0 {
                break;
            }
            count += len;
            buf = &mut buf[len..];
        }
        Ok((o_buf, count))
    }
    fn write_at(&self, offset: u64, buf: &RRefVec<u8>) -> VfsResult<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        let mut file = self.file.lock();
        if offset > *self.size.lock() {
            let empty = vec![0; (offset - *self.size.lock()) as usize];
            file.seek(fatfs::SeekFrom::Start(*self.size.lock()))
                .map_err(|_| VfsError::IoError)?;
            file.write_all(&empty).map_err(|_| VfsError::NoSpace)?;
        }
        let fat_offset = file.offset();
        if offset != fat_offset as u64 {
            file.seek(fatfs::SeekFrom::Start(offset))
                .map_err(|_| VfsError::IoError)?;
        }
        file.write_all(buf.as_slice())
            .map_err(|_| VfsError::NoSpace)?;
        if offset + buf.len() as u64 > *self.size.lock() {
            *self.size.lock() = offset + buf.len() as u64;
        }
        Ok(buf.len())
    }
    fn ioctl(&self, _cmd: u32, _arg: usize) -> VfsResult<usize> {
        Err(VfsError::NoTTY)
    }
    fn flush(&self) -> VfsResult<()> {
        self.fsync()
    }
    fn fsync(&self) -> VfsResult<()> {
        self.file.lock().flush().map_err(|_| VfsError::IoError)
    }
}

impl<R: VfsRawMutex + 'static> VfsInode for FatFsFileInode<R> {
    fn get_super_block(&self) -> VfsResult<Arc<dyn VfsSuperBlock>> {
        let sb = self.attr.sb.upgrade().unwrap();
        Ok(sb)
    }

    fn node_perm(&self) -> VfsNodePerm {
        self.attr.inner.lock().perm
    }

    fn set_attr(&self, _attr: InodeAttr) -> VfsResult<()> {
        Ok(())
    }

    fn get_attr(&self) -> VfsResult<VfsFileStat> {
        let attr = self.attr.inner.lock();
        let mode = VfsInodeMode::from(attr.perm, VfsNodeType::File).bits();
        let len = *self.size.lock();
        Ok(VfsFileStat {
            st_dev: 0,
            st_ino: 1,
            st_mode: mode,
            st_nlink: 1,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            __pad: 0,
            st_size: len,
            st_blksize: 512,
            __pad2: 0,
            st_blocks: len / 512,
            st_atime: attr.atime,
            st_mtime: attr.mtime,
            st_ctime: attr.ctime,
            unused: 0,
        })
    }
    fn list_xattr(&self) -> VfsResult<Vec<String>> {
        Err(VfsError::NoSys)
    }

    fn inode_type(&self) -> VfsNodeType {
        VfsNodeType::File
    }

    impl_file_inode_default!();

    fn truncate(&self, len: u64) -> VfsResult<()> {
        let mut this_len = self.size.lock();
        if *this_len == len {
            return Ok(());
        }
        let mut file = self.file.lock();
        file.seek(fatfs::SeekFrom::Start(len))
            .map_err(|_| VfsError::IoError)?;
        file.truncate().map_err(|_| VfsError::IoError)?;
        *this_len = len;
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
