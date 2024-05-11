use alloc::sync::Arc;
use core::fmt::Debug;

use constants::{
    io::{OpenFlags, PollEvents, SeekFrom},
    AlienError, AlienResult, LinuxErrno,
};
use interface::{NetDomain, SocketID};
use ksync::Mutex;
use rref::RRefVec;
use vfscore::{
    dentry::VfsDentry,
    error::VfsError,
    file::VfsFile,
    inode::VfsInode,
    utils::{VfsFileStat, VfsNodeType},
};

use crate::kfile::File;

pub struct SocketFile {
    net_stack_domain: Arc<dyn NetDomain>,
    pos: Mutex<u64>,
    open_flag: Mutex<OpenFlags>,
    socket_id: SocketID,
    inode: Arc<dyn VfsInode>,
}

impl Drop for SocketFile {
    fn drop(&mut self) {
        self.net_stack_domain
            .remove_socket(self.socket_id)
            .expect("remove socket failed")
    }
}

impl Debug for SocketFile {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SocketFile")
            .field("socket_id", &self.socket_id)
            .field("open_flag", &self.open_flag)
            .field("pos", &self.pos)
            .finish()
    }
}

impl SocketFile {
    pub fn new(
        net_stack_domain: Arc<dyn NetDomain>,
        socket_id: SocketID,
        open_flag: OpenFlags,
    ) -> Self {
        Self {
            net_stack_domain,
            pos: Mutex::new(0),
            open_flag: Mutex::new(open_flag),
            socket_id,
            inode: Arc::new(SocketInode),
        }
    }

    pub fn socket_id(&self) -> SocketID {
        self.socket_id
    }
}

impl File for SocketFile {
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
        let pos = *self.pos.lock();
        let write = self.write_at(pos, buf)?;
        *self.pos.lock() += write as u64;
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
        let shared_buf = RRefVec::from_slice(buf);
        let (shared_buf, len) =
            self.net_stack_domain
                .read_at(self.socket_id, offset, shared_buf)?;
        buf[..len].copy_from_slice(&shared_buf.as_slice()[..len]);
        Ok(len)
    }

    fn write_at(&self, offset: u64, buf: &[u8]) -> AlienResult<usize> {
        if buf.len() == 0 {
            return Ok(0);
        }
        let open_flag = self.open_flag.lock();
        if !open_flag.contains(OpenFlags::O_WRONLY) && !open_flag.contains(OpenFlags::O_RDWR) {
            return Err(LinuxErrno::EPERM);
        }
        let shared_buf = RRefVec::from_slice(buf);
        let write = self
            .net_stack_domain
            .write_at(self.socket_id, offset, &shared_buf)?;
        Ok(write)
    }

    fn flush(&self) -> AlienResult<()> {
        Ok(())
    }

    fn fsync(&self) -> AlienResult<()> {
        Ok(())
    }

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

    fn get_attr(&self) -> AlienResult<VfsFileStat> {
        Err(AlienError::ENOSYS)
    }
    fn set_open_flag(&self, _flag: OpenFlags) {
        panic!("SocketFile can't set open flag")
    }

    fn get_open_flag(&self) -> OpenFlags {
        *self.open_flag.lock()
    }

    fn dentry(&self) -> Arc<dyn VfsDentry> {
        panic!("SocketFile has no dentry")
    }
    fn inode(&self) -> Arc<dyn VfsInode> {
        panic!("SocketFile has no inode")
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

    fn poll(&self, event: PollEvents) -> AlienResult<PollEvents> {
        self.net_stack_domain.poll(self.socket_id, event)
    }
}

pub struct SocketInode;

impl VfsFile for SocketInode {}

impl VfsInode for SocketInode {
    fn inode_type(&self) -> VfsNodeType {
        VfsNodeType::Socket
    }
}
