use alloc::{sync::Arc, vec::Vec};

use unifs::inode::basic_file_stat;
use vfscore::{
    error::VfsError,
    file::VfsFile,
    impl_file_inode_default,
    inode::{InodeAttr, VfsInode},
    superblock::VfsSuperBlock,
    utils::{
        VfsFileStat, VfsInodeMode, VfsNodePerm, VfsNodeType, VfsPollEvents, VfsRenameFlag, VfsTime,
        VfsTimeSpec,
    },
    DVec, VfsResult,
};

use super::*;
use crate::RamFsProvider;
pub struct RamFsFileInode<T: Send + Sync, R: VfsRawMutex> {
    basic: UniFsInodeSame<T, R>,
    inner: lock_api::Mutex<R, RamFsFileInodeInner>,
    ext_attr: lock_api::Mutex<R, BTreeMap<String, String>>,
}
struct RamFsFileInodeInner {
    data: Vec<u8>,
}

impl<T: RamFsProvider + 'static, R: VfsRawMutex + 'static> RamFsFileInode<T, R> {
    pub fn new(
        sb: &Arc<UniFsSuperBlock<R>>,
        provider: T,
        inode_number: u64,
        perm: VfsNodePerm,
    ) -> Self {
        Self {
            basic: UniFsInodeSame::new(sb, provider, inode_number, perm),
            inner: lock_api::Mutex::new(RamFsFileInodeInner { data: Vec::new() }),
            ext_attr: lock_api::Mutex::new(BTreeMap::new()),
        }
    }
    pub fn update_metadata<F, Res>(&self, f: F) -> Res
    where
        F: FnOnce(&UniFsInodeSame<T, R>) -> Res,
    {
        f(&self.basic)
    }
}

impl<T: RamFsProvider + 'static, R: VfsRawMutex + 'static> VfsFile for RamFsFileInode<T, R> {
    fn read_at(&self, offset: u64, buf: DVec<u8>) -> VfsResult<(DVec<u8>, usize)> {
        let inner = self.inner.lock();
        let size = inner.data.len() as u64;
        let offset = offset.min(size);
        let len = (size - offset).min(buf.len() as u64) as usize;
        let data = inner.data.as_slice();
        let mut buf = buf;
        buf.as_mut_slice()[..len].copy_from_slice(&data[offset as usize..offset as usize + len]);
        Ok((buf, len))
    }
    fn write_at(&self, offset: u64, buf: &DVec<u8>) -> VfsResult<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        let mut inner = self.inner.lock();
        let buf_len = buf.len();
        let offset = offset as usize;
        let content = &mut inner.data;
        if offset + buf_len > content.len() {
            content.resize(offset + buf_len, 0);
        }
        let dst = &mut content[offset..offset + buf_len];
        dst.copy_from_slice(&buf.as_slice()[..dst.len()]);
        Ok(buf.len())
    }
    fn poll(&self, _event: VfsPollEvents) -> VfsResult<VfsPollEvents> {
        todo!()
    }
    fn ioctl(&self, _cmd: u32, _arg: usize) -> VfsResult<usize> {
        // let cmd = pconst::io::TeletypeCommand::try_from(cmd).map_err(|_| VfsError::Invalid)?;
        // warn!("not support ioctl, cmd: {:?}, arg: {:x}", cmd, arg);
        Err(VfsError::NoTTY)
    }
}

impl<T: RamFsProvider + 'static, R: VfsRawMutex + 'static> VfsInode for RamFsFileInode<T, R> {
    fn get_super_block(&self) -> VfsResult<Arc<dyn VfsSuperBlock>> {
        let res = self.basic.sb.upgrade().unwrap();
        Ok(res)
    }

    fn node_perm(&self) -> VfsNodePerm {
        self.basic.inner.lock().perm
    }

    fn set_attr(&self, attr: InodeAttr) -> VfsResult<()> {
        set_attr(&self.basic, attr);
        Ok(())
    }

    fn get_attr(&self) -> VfsResult<VfsFileStat> {
        let basic = &self.basic;
        let mut stat = basic_file_stat(basic);
        stat.st_size = self.inner.lock().data.len() as u64;
        stat.st_mode = VfsInodeMode::from(
            VfsNodePerm::from_bits_truncate(stat.st_mode as u16),
            VfsNodeType::File,
        )
        .bits();
        Ok(stat)
    }

    fn list_xattr(&self) -> VfsResult<Vec<String>> {
        let res = self.ext_attr.lock().keys().cloned().collect();
        Ok(res)
    }
    fn inode_type(&self) -> VfsNodeType {
        VfsNodeType::File
    }

    fn truncate(&self, len: u64) -> VfsResult<()> {
        let mut inner = self.inner.lock();
        if len < inner.data.len() as u64 {
            inner.data.truncate(len as _);
        } else {
            inner.data.resize(len as _, 0);
        }
        Ok(())
    }
    impl_file_inode_default!();
    fn update_time(&self, time: VfsTime, now: VfsTimeSpec) -> VfsResult<()> {
        match time {
            VfsTime::ModifiedTime(t) => self.basic.inner.lock().mtime = t,
            VfsTime::AccessTime(t) => self.basic.inner.lock().atime = t,
        }
        self.basic.inner.lock().ctime = now;
        Ok(())
    }
}
