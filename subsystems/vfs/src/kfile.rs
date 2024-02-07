use alloc::sync::Arc;
use constants::io::{Dirent64, DirentType, OpenFlags, PollEvents, SeekFrom};
use constants::AlienResult;
use constants::LinuxErrno;
use core::fmt::{Debug, Formatter};
use downcast_rs::{impl_downcast, DowncastSync};
use ksync::Mutex;
use vfscore::dentry::VfsDentry;
use vfscore::error::VfsError;
use vfscore::inode::VfsInode;
use vfscore::path::VfsPath;
use vfscore::utils::{VfsFileStat, VfsNodeType, VfsPollEvents};

pub struct KernelFile {
    pos: Mutex<u64>,
    open_flag: Mutex<OpenFlags>,
    dentry: Arc<dyn VfsDentry>,
}

impl Debug for KernelFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("KernelFile")
            .field("pos", &self.pos)
            .field("open_flag", &self.open_flag)
            .field("name", &self.dentry.name())
            .finish()
    }
}

impl KernelFile {
    pub fn new(dentry: Arc<dyn VfsDentry>, open_flag: OpenFlags) -> Self {
        let pos = if open_flag.contains(OpenFlags::O_APPEND) {
            dentry.inode().unwrap().get_attr().unwrap().st_size
        } else {
            0
        };
        Self {
            pos: Mutex::new(pos),
            open_flag: Mutex::new(open_flag),
            dentry,
        }
    }
}

pub trait File: DowncastSync + Debug {
    fn read(&self, buf: &mut [u8]) -> AlienResult<usize>;
    fn write(&self, buf: &[u8]) -> AlienResult<usize>;
    fn read_at(&self, _offset: u64, _buf: &mut [u8]) -> AlienResult<usize> {
        Err(LinuxErrno::ENOSYS)
    }
    fn write_at(&self, _offset: u64, _buf: &[u8]) -> AlienResult<usize> {
        Err(LinuxErrno::ENOSYS)
    }
    fn flush(&self) -> AlienResult<()> {
        Ok(())
    }
    fn fsync(&self) -> AlienResult<()> {
        Ok(())
    }
    fn seek(&self, pos: SeekFrom) -> AlienResult<u64>;
    /// Gets the file attributes.
    fn get_attr(&self) -> AlienResult<VfsFileStat>;
    fn ioctl(&self, _cmd: u32, _arg: usize) -> AlienResult<usize> {
        Err(LinuxErrno::ENOSYS)
    }
    fn set_open_flag(&self, _flag: OpenFlags) {}
    fn get_open_flag(&self) -> OpenFlags {
        OpenFlags::O_RDONLY
    }
    fn dentry(&self) -> Arc<dyn VfsDentry>;
    fn inode(&self) -> Arc<dyn VfsInode>;
    fn readdir(&self, _buf: &mut [u8]) -> AlienResult<usize> {
        Err(LinuxErrno::ENOSYS)
    }
    fn truncate(&self, _len: u64) -> AlienResult<()> {
        Err(LinuxErrno::ENOSYS)
    }
    fn is_readable(&self) -> bool;
    fn is_writable(&self) -> bool;
    fn is_append(&self) -> bool;
    fn poll(&self, _event: PollEvents) -> AlienResult<PollEvents> {
        Err(LinuxErrno::ENOSYS)
    }
}

impl_downcast!(sync  File);

// todo! permission check
impl File for KernelFile {
    fn read(&self, buf: &mut [u8]) -> AlienResult<usize> {
        if buf.len() == 0 {
            return Ok(0);
        }
        let pos = *self.pos.lock();
        let read = self.read_at(pos, buf)?;
        *self.pos.lock() += read as u64;
        Ok(read)
    }
    fn write(&self, buf: &[u8]) -> AlienResult<usize> {
        if buf.len() == 0 {
            return Ok(0);
        }
        let mut pos = self.pos.lock();
        let write = self.write_at(*pos, buf)?;
        *pos += write as u64;
        Ok(write)
    }
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> AlienResult<usize> {
        if buf.len() == 0 {
            return Ok(0);
        }
        let open_flag = self.open_flag.lock();
        if !open_flag.contains(OpenFlags::O_RDONLY) && !open_flag.contains(OpenFlags::O_RDWR) {
            return Err(LinuxErrno::EPERM);
        }
        drop(open_flag);
        let inode = self.dentry.inode()?;
        let read = inode.read_at(offset, buf)?;
        Ok(read)
    }

    fn write_at(&self, offset: u64, buf: &[u8]) -> AlienResult<usize> {
        if buf.len() == 0 {
            return Ok(0);
        }
        let open_flag = self.open_flag.lock();
        if !open_flag.contains(OpenFlags::O_WRONLY) && !open_flag.contains(OpenFlags::O_RDWR) {
            return Err(LinuxErrno::EPERM);
        }
        let inode = self.dentry.inode()?;
        let write = inode.write_at(offset, buf)?;
        Ok(write)
    }

    fn flush(&self) -> AlienResult<()> {
        let open_flag = self.open_flag.lock();
        if !open_flag.contains(OpenFlags::O_WRONLY) & !open_flag.contains(OpenFlags::O_RDWR) {
            return Err(LinuxErrno::EPERM);
        }
        let inode = self.dentry.inode()?;
        inode.flush()?;
        Ok(())
    }

    fn fsync(&self) -> AlienResult<()> {
        let open_flag = self.open_flag.lock();
        if !open_flag.contains(OpenFlags::O_WRONLY) && !open_flag.contains(OpenFlags::O_RDWR) {
            return Err(LinuxErrno::EPERM);
        }
        let inode = self.dentry.inode()?;
        inode.fsync()?;
        Ok(())
    }

    // check for special file
    fn seek(&self, pos: SeekFrom) -> AlienResult<u64> {
        let mut spos = self.pos.lock();
        let size = self.get_attr()?.st_size;
        let new_offset = match pos {
            SeekFrom::Start(pos) => Some(pos),
            SeekFrom::Current(off) => spos.checked_add_signed(off),
            SeekFrom::End(off) => size.checked_add_signed(off),
        }
        .ok_or_else(|| VfsError::Invalid)?;
        *spos = new_offset;
        Ok(new_offset)
    }

    /// Gets the file attributes.
    fn get_attr(&self) -> AlienResult<VfsFileStat> {
        self.dentry.inode()?.get_attr().map_err(Into::into)
    }

    fn ioctl(&self, _cmd: u32, _arg: usize) -> AlienResult<usize> {
        let inode = self.dentry.inode().unwrap();
        inode.ioctl(_cmd, _arg).map_err(Into::into)
    }

    fn set_open_flag(&self, flag: OpenFlags) {
        *self.open_flag.lock() = flag;
    }

    fn get_open_flag(&self) -> OpenFlags {
        *self.open_flag.lock()
    }
    fn dentry(&self) -> Arc<dyn VfsDentry> {
        self.dentry.clone()
    }
    fn inode(&self) -> Arc<dyn VfsInode> {
        self.dentry.inode().unwrap()
    }
    fn readdir(&self, buf: &mut [u8]) -> AlienResult<usize> {
        let inode = self.inode();
        let mut pos = self.pos.lock();
        let mut count = 0;
        let mut ptr = buf.as_mut_ptr();
        loop {
            let dirent = inode.readdir(*pos as usize).map_err(|e| {
                *pos = 0;
                e
            })?;
            match dirent {
                Some(d) => {
                    let dirent64 =
                        Dirent64::new(&d.name, d.ino, *pos as i64, vfsnodetype2dirent64(d.ty));
                    if count + dirent64.len() <= buf.len() {
                        let dirent_ptr = unsafe { &mut *(ptr as *mut Dirent64) };
                        *dirent_ptr = dirent64;
                        let name_ptr = dirent_ptr.name.as_mut_ptr();
                        unsafe {
                            let mut name = d.name.clone();
                            name.push('\0');
                            let len = name.len();
                            name_ptr.copy_from(name.as_ptr(), len);
                            ptr = ptr.add(dirent_ptr.len());
                        }
                        count += dirent_ptr.len();
                    } else {
                        break;
                    } // Buf is small
                }
                None => {
                    break;
                } // EOF
            }
            *pos += 1;
        }
        Ok(count)
    }
    fn truncate(&self, len: u64) -> AlienResult<()> {
        let open_flag = self.open_flag.lock();
        if !open_flag.contains(OpenFlags::O_WRONLY) & !open_flag.contains(OpenFlags::O_RDWR) {
            return Err(LinuxErrno::EINVAL);
        }
        let dt = self.dentry();
        VfsPath::new(dt).truncate(len).map_err(Into::into)
    }
    fn is_readable(&self) -> bool {
        let open_flag = self.open_flag.lock();
        open_flag.contains(OpenFlags::O_RDONLY) | open_flag.contains(OpenFlags::O_RDWR)
    }
    fn is_writable(&self) -> bool {
        let open_flag = self.open_flag.lock();
        open_flag.contains(OpenFlags::O_WRONLY) | open_flag.contains(OpenFlags::O_RDWR)
    }

    fn is_append(&self) -> bool {
        let open_flag = self.open_flag.lock();
        open_flag.contains(OpenFlags::O_APPEND)
    }

    fn poll(&self, _event: PollEvents) -> AlienResult<PollEvents> {
        let inode = self.dentry.inode()?;
        let res = inode
            .poll(VfsPollEvents::from_bits_truncate(_event.bits()))
            .map(|e| PollEvents::from_bits_truncate(e.bits()));
        res.map_err(Into::into)
    }
}

fn vfsnodetype2dirent64(ty: VfsNodeType) -> DirentType {
    DirentType::from_u8(ty as u8)
}

impl Drop for KernelFile {
    fn drop(&mut self) {
        let _ = self.flush();
        let _ = self.fsync();
    }
}
