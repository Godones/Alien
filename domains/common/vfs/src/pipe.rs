use alloc::{
    string::String,
    sync::{Arc, Weak},
    vec::Vec,
};
use core::fmt::{Debug, Formatter};

use basic::{
    config::PIPE_BUF,
    constants::io::{OpenFlags, PollEvents, SeekFrom},
    sync::Mutex,
    AlienError, AlienResult,
};
use log::debug;
use vfscore::{
    dentry::VfsDentry,
    error::VfsError,
    file::VfsFile,
    impl_common_inode_default,
    inode::{InodeAttr, VfsInode},
    superblock::VfsSuperBlock,
    utils::{
        VfsFileStat, VfsNodePerm, VfsNodeType, VfsPollEvents, VfsRenameFlag, VfsTime, VfsTimeSpec,
    },
    VfsResult,
};

use crate::{kfile::File, SCHEDULER_DOMAIN};
pub struct PipeFile {
    open_flag: Mutex<OpenFlags>,
    inode_copy: Arc<PipeInode>,
}

impl Debug for PipeFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PipeFile")
            .field("open_flag", &self.open_flag)
            .finish()
    }
}

impl PipeFile {
    pub fn new(open_flag: OpenFlags, inode_copy: Arc<PipeInode>) -> Self {
        Self {
            open_flag: Mutex::new(open_flag),
            inode_copy,
        }
    }
}

impl File for PipeFile {
    fn read(&self, buf: &mut [u8]) -> AlienResult<usize> {
        if buf.len() == 0 {
            return Ok(0);
        }
        self.inode_copy.read_at(0, buf).map_err(|e| e.into())
    }
    fn write(&self, buf: &[u8]) -> AlienResult<usize> {
        if buf.len() == 0 {
            return Ok(0);
        }
        self.inode_copy.write_at(0, buf).map_err(|e| e.into())
    }
    fn read_at(&self, _offset: u64, buf: &mut [u8]) -> AlienResult<usize> {
        self.read(buf)
    }
    fn write_at(&self, _offset: u64, buf: &[u8]) -> AlienResult<usize> {
        self.write(buf)
    }

    fn flush(&self) -> AlienResult<()> {
        Ok(())
    }

    fn fsync(&self) -> AlienResult<()> {
        Ok(())
    }

    fn seek(&self, _pos: SeekFrom) -> AlienResult<u64> {
        Err(AlienError::ESPIPE)
    }

    fn get_attr(&self) -> AlienResult<VfsFileStat> {
        Err(AlienError::ENOSYS)
    }

    fn set_open_flag(&self, flag: OpenFlags) {
        *self.open_flag.lock() = flag;
    }
    fn get_open_flag(&self) -> OpenFlags {
        *self.open_flag.lock()
    }
    fn dentry(&self) -> Arc<dyn VfsDentry> {
        panic!("PipeFile has no dentry")
    }
    fn inode(&self) -> Arc<dyn VfsInode> {
        self.inode_copy.clone()
    }
    fn is_readable(&self) -> bool {
        let open_flag = self.open_flag.lock();
        open_flag.contains(OpenFlags::O_RDONLY | OpenFlags::O_RDWR)
    }

    fn is_writable(&self) -> bool {
        let open_flag = self.open_flag.lock();
        open_flag.contains(OpenFlags::O_WRONLY | OpenFlags::O_RDWR)
    }
    fn is_append(&self) -> bool {
        false
    }
    fn poll(&self, _event: PollEvents) -> AlienResult<PollEvents> {
        let res = self
            .inode_copy
            .poll(VfsPollEvents::from_bits_truncate(_event.bits()))
            .map(|e| PollEvents::from_bits_truncate(e.bits()));
        res.map_err(Into::into)
    }
}

pub struct PipeInode {
    data: Mutex<PipeInodeData>,
}

struct PipeInodeData {
    /// 缓冲区的数据部分
    pub buf: [u8; PIPE_BUF],
    /// 缓冲区头部，用于指明当前的读位置
    pub head: usize,
    /// 缓冲区尾部，用于指明当前的写位置
    pub tail: usize,
    /// 记录 在 读端 进行等待的进程
    pub read_wait: Option<Weak<PipeFile>>,
    /// 记录 在 写端 进行等待的进程
    pub write_wait: Option<Weak<PipeFile>>,
}

impl PipeInodeData {
    /// 用于返回当前的缓冲区是否为空
    pub fn is_empty(&self) -> bool {
        self.head == self.tail
    }

    /// 用于返回当前的缓冲区是否为满
    pub fn is_full(&self) -> bool {
        (self.tail + 1) % PIPE_BUF == self.head
    }

    /// 返回当前缓冲区中能够被读的字节数
    pub fn available_read(&self) -> usize {
        if self.head <= self.tail {
            self.tail - self.head
        } else {
            PIPE_BUF - self.head + self.tail
        }
    }
    /// 返回当前缓冲区中还能够写入的字节数
    pub fn available_write(&self) -> usize {
        if self.head <= self.tail {
            PIPE_BUF - self.tail + self.head - 1
        } else {
            self.head - self.tail - 1
        }
    }

    /// 向缓冲区中写入数据，返回写入的字节数
    pub fn write(&mut self, buf: &[u8]) -> usize {
        let mut count = 0;
        while !self.is_full() && count < buf.len() {
            self.buf[self.tail] = buf[count];
            self.tail = (self.tail + 1) % PIPE_BUF;
            count += 1;
        }
        count
    }

    /// 从缓冲区中读取数据，返回读取的字节数
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        let mut count = 0;
        while !self.is_empty() && count < buf.len() {
            buf[count] = self.buf[self.head];
            self.head = (self.head + 1) % PIPE_BUF;
            count += 1;
        }
        count
    }

    /// 返回是否有进程在 写端等待
    pub fn is_write_wait(&self) -> bool {
        self.write_wait.is_some() && self.write_wait.as_ref().unwrap().upgrade().is_some()
    }

    /// 返回是否有进程在 读端等待
    pub fn is_read_wait(&self) -> bool {
        self.read_wait.is_some() && self.read_wait.as_ref().unwrap().upgrade().is_some()
    }
}

impl PipeInode {
    /// 创建一片新的管道缓冲区，在 `Pipe::new` 中被调用
    pub fn new() -> PipeInode {
        PipeInode {
            data: Mutex::new(PipeInodeData {
                buf: [0; PIPE_BUF],
                head: 0,
                tail: 0,
                read_wait: None,
                write_wait: None,
            }),
        }
    }

    pub fn set_reader(&self, reader: &Arc<PipeFile>) {
        let mut data = self.data.lock();
        data.read_wait = Some(Arc::downgrade(reader))
    }
    pub fn set_sender(&self, sender: &Arc<PipeFile>) {
        let mut data = self.data.lock();
        data.write_wait = Some(Arc::downgrade(sender))
    }
}

impl VfsFile for PipeInode {
    fn read_at(&self, _offset: u64, user_buf: &mut [u8]) -> VfsResult<usize> {
        debug!("pipe_read: user_buf's len {:?}", user_buf.len());
        let mut count = 0;
        loop {
            let mut buf = self.data.lock();
            let available = buf.available_read();
            debug!("pipe_read: available:{}", available);
            if available == 0 {
                if !buf.is_write_wait() {
                    // if there is no process waiting for writing, we should return
                    break;
                } else {
                    // wait for writing
                    drop(buf);
                    // do_suspend();
                    SCHEDULER_DOMAIN.get().unwrap().yield_now().unwrap();
                    debug!("pipe_read: suspend");
                }
            } else {
                let min = core::cmp::min(available, user_buf.len() - count);
                count += buf.read(&mut user_buf[count..count + min]);
                break;
            }
        }
        debug!("pipe_read: return count:{}", count);
        Ok(count)
    }
    fn write_at(&self, _offset: u64, user_buf: &[u8]) -> VfsResult<usize> {
        debug!("pipe_write: {:?}", user_buf.len());
        let mut count = 0;
        loop {
            let mut buf = self.data.lock();
            let available = buf.available_write();
            if available == 0 {
                if !buf.is_read_wait() {
                    // if there is no process waiting for reading, we should return
                    break;
                }
                // release lock
                drop(buf);
                // wait for reading
                // do_suspend();
                SCHEDULER_DOMAIN.get().unwrap().yield_now().unwrap();
            } else {
                let min = core::cmp::min(available, user_buf.len() - count);
                debug!("pipe_write: min:{}, count:{}", min, count);
                count += buf.write(&user_buf[count..count + min]);
                break;
            }
        }
        debug!("pipe_write: count:{}", count);
        Ok(count)
    }
    fn poll(&self, event: VfsPollEvents) -> VfsResult<VfsPollEvents> {
        let data = self.data.lock();
        let mut res = VfsPollEvents::empty();
        if event.contains(VfsPollEvents::IN) {
            if data.available_read() > 0 {
                res |= VfsPollEvents::IN;
            }
        }
        if event.contains(VfsPollEvents::OUT) {
            if data.available_write() > 0 {
                res |= VfsPollEvents::OUT
            }
        }
        let is_reader = data.is_read_wait();
        let is_sender = data.is_write_wait();
        if is_reader && !is_sender {
            res |= VfsPollEvents::HUP;
        }
        if !is_reader && is_sender {
            res |= VfsPollEvents::ERR;
        }
        Ok(res)
    }
}

impl VfsInode for PipeInode {
    impl_common_inode_default!();

    fn get_super_block(&self) -> VfsResult<Arc<dyn VfsSuperBlock>> {
        Err(VfsError::NoSys)
    }

    fn node_perm(&self) -> VfsNodePerm {
        VfsNodePerm::empty()
    }

    fn readlink(&self, _buf: &mut [u8]) -> VfsResult<usize> {
        Err(VfsError::NoSys)
    }

    fn set_attr(&self, _attr: InodeAttr) -> VfsResult<()> {
        Err(VfsError::NoSys)
    }

    fn get_attr(&self) -> VfsResult<VfsFileStat> {
        Err(VfsError::NoSys)
    }

    fn list_xattr(&self) -> VfsResult<Vec<String>> {
        Err(VfsError::NoSys)
    }

    fn inode_type(&self) -> VfsNodeType {
        VfsNodeType::Fifo
    }
    fn update_time(&self, _: VfsTime, _: VfsTimeSpec) -> VfsResult<()> {
        Err(VfsError::NoSys)
    }
}

impl Drop for PipeFile {
    fn drop(&mut self) {
        let data = self.inode_copy.data.lock();
        let is_reader = data.is_read_wait();
        let is_sender = data.is_write_wait();
        if !is_reader && !is_sender {}
    }
}

pub fn make_pipe_file() -> (Arc<PipeFile>, Arc<PipeFile>) {
    let inode = Arc::new(PipeInode::new());
    let reader = Arc::new(PipeFile::new(OpenFlags::O_RDONLY, inode.clone()));
    let sender = Arc::new(PipeFile::new(OpenFlags::O_WRONLY, inode.clone()));
    inode.set_reader(&reader);
    inode.set_sender(&sender);
    (reader, sender)
}
