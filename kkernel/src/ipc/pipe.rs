//! 管道是一种最基本的IPC机制，作用于有血缘关系的进程之间，完成数据传递。
//!
//! `Alien` 中对于管道的设计参考了`rCore`的相关设计。创建管道时会同时创建一个环形缓冲区，
//! 管道的两个端口抽象成文件，对两个端口直接的相关的文件操作（读操作或者写操作）都被设计
//! 成对缓冲区进行数据处理（向缓冲区中传入数据或接收数据）。
//!
//! 管道文件创建时，依据 Alien 所使用的 rvfs 中对文件 `File` 的规定，我们只需为管道文件规定好
//! [`pipe_release`]、[`pipe_write`]、[`pipe_read`]、[`pipe_exec`]、[`pipe_llseek`]、
//! [`pipe_read_is_hang_up`]、[`pipe_write_is_hang_up`]、[`pipe_ready_to_read`]
//! 、[`pipe_ready_to_write`] 几个操作函数，即可快速的创建管道文件，并将其放入进程的文件描述
//! 符表中。

use config::PIPE_BUF;
use crate::fs::file::File;
use crate::task::{current_task, do_suspend};
use alloc::string::{String, ToString};
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use constants::io::{OpenFlags, PollEvents, SeekFrom};
use constants::AlienResult;
use constants::LinuxErrno;
use core::fmt::{Debug, Formatter};
use core::sync::atomic::AtomicUsize;
use ksync::Mutex;
use vfscore::dentry::VfsDentry;
use vfscore::error::VfsError;
use vfscore::file::VfsFile;
use vfscore::impl_common_inode_default;
use vfscore::inode::{InodeAttr, VfsInode};
use vfscore::superblock::VfsSuperBlock;
use vfscore::utils::VfsPollEvents;
use vfscore::utils::*;
use vfscore::VfsResult;
use vfs::pipefs::{PIPE_FS_ROOT, PipeFsDirInodeImpl};

static PIPE: AtomicUsize = AtomicUsize::new(0);

/// 管道文件
pub struct PipeFile {
    open_flag: Mutex<OpenFlags>,
    dentry: Arc<dyn VfsDentry>,
    inode_copy: Arc<PipeInode>,
}

impl Debug for PipeFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PipeFile")
            .field("open_flag", &self.open_flag)
            .field("name", &self.dentry.name())
            .finish()
    }
}

impl PipeFile {
    pub fn new(
        dentry: Arc<dyn VfsDentry>,
        open_flag: OpenFlags,
        inode_copy: Arc<PipeInode>,
    ) -> Self {
        Self {
            open_flag: Mutex::new(open_flag),
            dentry,
            inode_copy,
        }
    }
}

/// create a pipe file
pub fn make_pipe_file() -> VfsResult<(Arc<PipeFile>, Arc<PipeFile>)> {
    let root = PIPE_FS_ROOT.get().unwrap();
    let root_inode = root
        .inode()?
        .downcast_arc::<PipeFsDirInodeImpl>()
        .map_err(|_| VfsError::Invalid)
        .unwrap();
    let inode = Arc::new(PipeInode::new());
    let num_str = PIPE.fetch_add(1, core::sync::atomic::Ordering::AcqRel);
    let same_inode =
        root_inode.add_file_manually(&num_str.to_string(), inode.clone(), "rw-rw-rw-".into())?;
    let dt = root.i_insert(&num_str.to_string(), same_inode)?;
    let reader = Arc::new(PipeFile::new(
        dt.clone(),
        OpenFlags::O_RDONLY,
        inode.clone(),
    ));
    let sender = Arc::new(PipeFile::new(dt, OpenFlags::O_WRONLY, inode.clone()));
    inode.set_reader(&reader);
    inode.set_sender(&sender);
    Ok((reader, sender))
}

impl File for PipeFile {
    fn read(&self, buf: &mut [u8]) -> AlienResult<usize> {
        if buf.len() == 0 {
            return Ok(0);
        }
        self.dentry.inode()?.read_at(0, buf).map_err(|e| e.into())
    }
    fn write(&self, buf: &[u8]) -> AlienResult<usize> {
        if buf.len() == 0 {
            return Ok(0);
        }
        self.dentry.inode()?.write_at(0, buf).map_err(|e| e.into())
    }
    fn seek(&self, _pos: SeekFrom) -> AlienResult<u64> {
        Err(LinuxErrno::ESPIPE)
    }
    fn get_attr(&self) -> AlienResult<VfsFileStat> {
        Err(LinuxErrno::ENOSYS)
    }
    fn dentry(&self) -> Arc<dyn VfsDentry> {
        self.dentry.clone()
    }
    fn inode(&self) -> Arc<dyn VfsInode> {
        self.dentry.inode().unwrap()
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
        let inode = self.dentry.inode()?;
        let res = inode
            .poll(VfsPollEvents::from_bits_truncate(_event.bits()))
            .map(|e| PollEvents::from_bits_truncate(e.bits()));
        res.map_err(Into::into)
    }
}

/// 环形缓冲区，用于在内存中维护管道的相关信息。
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
        info!("pipe_read: user_buf's len {:?}", user_buf.len());
        let mut count = 0;
        loop {
            let mut buf = self.data.lock();
            let available = buf.available_read();
            info!("pipe_read: available:{}", available);
            if available == 0 {
                if !buf.is_write_wait() {
                    // if there is no process waiting for writing, we should return
                    break;
                } else {
                    // wait for writing
                    drop(buf);
                    do_suspend();
                    warn!("pipe_read: suspend");
                    // check signal
                    let task = current_task().unwrap();
                    // interrupt by signal
                    let task_inner = task.access_inner();
                    let receiver = task_inner.signal_receivers.lock();
                    if receiver.have_signal() {
                        error!("pipe_write: have signal");
                        return Err(VfsError::EINTR);
                    }
                }
            } else {
                let min = core::cmp::min(available, user_buf.len() - count);
                count += buf.read(&mut user_buf[count..count + min]);
                break;
            }
        }
        info!("pipe_read: return count:{}", count);
        Ok(count)
    }
    fn write_at(&self, _offset: u64, user_buf: &[u8]) -> VfsResult<usize> {
        info!("pipe_write: {:?}", user_buf.len());
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
                do_suspend();
                let task = current_task().unwrap();
                let task_inner = task.access_inner();
                let receiver = task_inner.signal_receivers.lock();
                if receiver.have_signal() {
                    error!("pipe_write: have signal");
                    return Err(VfsError::EINTR);
                }
            } else {
                let min = core::cmp::min(available, user_buf.len() - count);
                info!("pipe_write: min:{}, count:{}", min, count);
                count += buf.write(&user_buf[count..count + min]);
                break;
            }
        }
        info!("pipe_write: count:{}", count);
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
        if !is_reader && !is_sender {
            let name = self.dentry.name();
            let root = PIPE_FS_ROOT.get().unwrap();
            let root_inode = root
                .inode()
                .unwrap()
                .downcast_arc::<PipeFsDirInodeImpl>()
                .map_err(|_| VfsError::Invalid)
                .unwrap();
            root.remove(&name).unwrap();
            root_inode.remove_manually(&name).unwrap();
        }
    }
}
