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

use alloc::boxed::Box;
use alloc::sync::{Arc, Weak};
use core::intrinsics::forget;

use rvfs::dentry::DirEntry;
use rvfs::file::{File, FileExtOps, FileMode, FileOps, OpenFlags, SeekFrom};
use rvfs::inode::SpecialData;
use rvfs::mount::VfsMount;
use rvfs::StrResult;

use crate::config::PIPE_BUF;
use crate::fs::file::KFile;
use crate::task::{current_task, do_suspend};

/// 管道结构
pub struct Pipe;

/// 环形缓冲区，用于在内存中维护管道的相关信息。
pub struct RingBuffer {
    /// 缓冲区的数据部分
    pub buf: [u8; PIPE_BUF],
    /// 缓冲区头部，用于指明当前的读位置
    pub head: usize,
    /// 缓冲区尾部，用于指明当前的写位置
    pub tail: usize,
    /// 记录 在 读端 进行等待的进程
    pub read_wait: Option<Weak<KFile>>,
    /// 记录 在 写端 进行等待的进程
    pub write_wait: Option<Weak<KFile>>,
    /// 引用计数
    pub ref_count: usize,
}

impl RingBuffer {
    /// 创建一片新的管道缓冲区，在 `Pipe::new` 中被调用
    pub fn new() -> RingBuffer {
        RingBuffer {
            buf: [0; PIPE_BUF],
            head: 0,
            tail: 0,
            read_wait: None,
            write_wait: None,
            ref_count: 2,
        }
    }

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

    /// 清除缓冲区中的内容，当前实现为将 head 和 tail 重置为 0
    pub fn clear(&mut self) {
        self.head = 0;
        self.tail = 0;
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

impl Pipe {
    /// 创建一个管道，初始化环形缓冲区，返回一对文件，分别对应读端文件和写端文件。
    /// 
    /// 过程包括创建一个环形缓冲区、创建并初始化写端文件和读端文件、
    /// 将两个文件与环形缓冲区相连、使得通过两个端文件快速对缓冲区进行读写等。
    pub fn new() -> (Arc<KFile>, Arc<KFile>) {
        let mut buf = Box::new(RingBuffer::new());
        let mut tx_file = File::new(
            Arc::new(DirEntry::empty()),
            Arc::new(VfsMount::empty()),
            OpenFlags::O_WRONLY,
            FileMode::FMODE_WRITE,
            FileOps::empty(),
        );
        tx_file.f_ops = {
            let mut ops = FileOps::empty();
            ops.write = pipe_write;
            ops.release = pipe_release;
            ops.llseek = pipe_llseek;
            ops
        };
        tx_file.access_inner().f_ops_ext = {
            FileExtOps {
                is_ready_read: |_| false,
                is_ready_write: pipe_ready_to_write,
                is_ready_exception: |_| false,
                is_hang_up: pipe_write_is_hang_up,
                ioctl: |_, _, _| -1,
            }
        };
        let mut rx_file = File::new(
            Arc::new(DirEntry::empty()),
            Arc::new(VfsMount::empty()),
            OpenFlags::O_RDONLY,
            FileMode::FMODE_READ,
            FileOps::empty(),
        );
        rx_file.f_ops = {
            let mut ops = FileOps::empty();
            ops.read = pipe_read;
            ops.release = pipe_release;
            ops.llseek = pipe_llseek;
            ops
        };
        rx_file.access_inner().f_ops_ext = {
            FileExtOps {
                is_ready_read: pipe_ready_to_read,
                is_ready_write: |_| false,
                is_ready_exception: |_| false,
                is_hang_up: pipe_read_is_hang_up,
                ioctl: |_, _, _| -1,
            }
        };

        let (rx_file, tx_file) = (Arc::new(rx_file), Arc::new(tx_file));
        let (rx_file, tx_file) = (KFile::new(rx_file), KFile::new(tx_file));
        buf.read_wait = Some(Arc::downgrade(&rx_file));
        buf.write_wait = Some(Arc::downgrade(&tx_file));
        let ptr = Box::into_raw(buf) as *const u8;
        rx_file
            .f_dentry
            .access_inner()
            .d_inode
            .access_inner()
            .special_data = Some(SpecialData::PipeData(ptr));
        tx_file
            .f_dentry
            .access_inner()
            .d_inode
            .access_inner()
            .special_data = Some(SpecialData::PipeData(ptr));
        (rx_file, tx_file)
    }
}

/// 管道文件的写操作，效果等同于 RingBuffer::write
fn pipe_write(file: Arc<File>, user_buf: &[u8], _offset: u64) -> StrResult<usize> {
    warn!("pipe_write: {:?}, mode:{:?}", user_buf.len(), file.f_mode);
    let inode = file.f_dentry.access_inner().d_inode.clone();
    let mut count = 0;
    loop {
        let inode_inner = inode.access_inner();
        assert!(inode_inner.special_data.is_some());
        let data = inode_inner.special_data.as_ref().unwrap();
        let ptr = match data {
            SpecialData::PipeData(ptr) => *ptr,
            _ => panic!("pipe_write: invalid special data"),
        };
        if ptr.is_null() {
            panic!("pipe_write: ptr is null");
        }
        let mut buf = unsafe { Box::from_raw(ptr as *mut RingBuffer) };
        let available = buf.available_write();
        if available == 0 {
            if !buf.is_read_wait() {
                // if there is no process waiting for reading, we should return
                forget(buf);
                break;
            }
            // wait for reading
            drop(inode_inner);
            do_suspend();
            let task = current_task().unwrap();
            let task_inner = task.access_inner();
            let receiver = task_inner.signal_receivers.lock();
            if receiver.have_signal() {
                error!("pipe_write: have signal");
                forget(buf);
                return Err("pipe_write: have signal");
            }
        } else {
            let min = core::cmp::min(available, user_buf.len() - count);
            error!("pipe_write: min:{}, count:{}", min, count);
            count += buf.write(&user_buf[count..count + min]);
            forget(buf);
            break;
        }
        forget(buf); // we can't drop the buf here, because the inode still holds the pointer
    }
    warn!("pipe_write: count:{}", count);
    Ok(count)
}

/// 管道文件的写操作，效果等同于 RingBuffer::read
fn pipe_read(file: Arc<File>, user_buf: &mut [u8], _offset: u64) -> StrResult<usize> {
    debug!("pipe_read: {:?}", user_buf.len());
    let inode = file.f_dentry.access_inner().d_inode.clone();
    let mut count = 0;
    loop {
        let inode_inner = inode.access_inner();
        assert!(inode_inner.special_data.is_some());
        let data = inode_inner.special_data.as_ref().unwrap();
        let ptr = match data {
            SpecialData::PipeData(ptr) => *ptr,
            _ => panic!("pipe_read: invalid special data"),
        };
        let mut buf = unsafe { Box::from_raw(ptr as *mut RingBuffer) };
        let available = buf.available_read();
        warn!("pipe_read: available:{}", available);
        if available == 0 {
            if !buf.is_write_wait() {
                // if there is no process waiting for writing, we should return
                forget(buf);
                break;
            }
            // wait for writing
            drop(inode_inner);
            do_suspend();
            error!("pipe_read: suspend");
            // check signal
            let task = current_task().unwrap();
            // interrupt by signal
            let task_inner = task.access_inner();
            let receiver = task_inner.signal_receivers.lock();
            if receiver.have_signal() {
                forget(buf);
                return Err("interrupted by signal");
            }
        } else {
            let min = core::cmp::min(available, user_buf.len() - count);
            count += buf.read(&mut user_buf[count..count + min]);
            forget(buf);
            break;
        }
        forget(buf); // we can't drop the buf here, because the inode still holds the pointer
    }
    warn!("pipe_read: return count:{}", count);
    Ok(count)
}

/// 管道文件的释放操作，用于关闭管道文件时
fn pipe_release(file: Arc<File>) -> StrResult<()> {
    warn!("pipe_release: file");
    assert_eq!(Arc::strong_count(&file), 1);
    let inode = &file.f_dentry.access_inner().d_inode;
    assert!(inode.access_inner().special_data.is_some());
    let inode_inner = inode.access_inner();
    let data = inode_inner.special_data.as_ref().unwrap();
    let ptr = match data {
        SpecialData::PipeData(ptr) => *ptr,
        _ => panic!("pipe_release: invalid special data"),
    };
    let mut buf = unsafe { Box::from_raw(ptr as *mut RingBuffer) };
    buf.ref_count -= 1;
    warn!("buf.refcount :{}", buf.ref_count);
    if buf.ref_count == 0 {
        // the last pipe file is closed, we should free the buffer
        debug!("pipe_release: free buffer");
        drop(buf);
    } else {
        forget(buf)
    }
    warn!("pipe_release: return");
    Ok(())
}

/// 管道文件 [`pipe_exec`] 操作中 `func` 参数的类型
pub enum PipeFunc {
    /// 读就绪操作
    AvailableRead,
    /// 写就绪操作
    AvailableWrite,
    /// 某端是否被悬挂操作，当其中的布尔值为 true 时，表示检查的是读端；否则为写端。具体可见 [`pipe_read_is_hang_up`] 和 [`pipe_write_is_hang_up`]
    Hangup(bool),
    /// 未知操作
    Unknown,
}

/// 管道文件的 exec 操作，用于被其他文件操作函数调用
fn pipe_exec(file: Arc<File>, func: PipeFunc) -> bool {
    let inode = file.f_dentry.access_inner().d_inode.clone();
    let inode_inner = inode.access_inner();
    assert!(inode_inner.special_data.is_some());
    let data = inode_inner.special_data.as_ref().unwrap();
    let ptr = match data {
        SpecialData::PipeData(ptr) => *ptr,
        _ => panic!("pipe_read: invalid special data"),
    };
    let buf = unsafe { Box::from_raw(ptr as *mut RingBuffer) };
    let res = match func {
        PipeFunc::AvailableRead => {
            let av = buf.available_read();
            trace!("pipe_exec: available_read:{}", av);
            av > 0
        }
        PipeFunc::AvailableWrite => buf.available_write() > 0,
        PipeFunc::Hangup(is_read) => {
            if is_read {
                !buf.is_write_wait()
            } else {
                !buf.is_read_wait()
            }
        }
        _ => false,
    };
    forget(buf);
    res
}

/// 管道文件的读就绪操作，用于检查管道文件是否准备好读，效果等同于 RingBuffer::available_read
fn pipe_ready_to_read(file: Arc<File>) -> bool {
    pipe_exec(file, PipeFunc::AvailableRead)
}

/// 管道文件的写就绪操作，用于检查管道文件是否准备好写，效果等同于 RingBuffer::available_write
fn pipe_ready_to_write(file: Arc<File>) -> bool {
    pipe_exec(file, PipeFunc::AvailableWrite)
}

/// 管道文件的 "读端是否处于悬挂状态" 操作，用于检查管道文件的写端是否已经被关闭，效果等同于 !RingBuffer::is_write_wait
fn pipe_read_is_hang_up(file: Arc<File>) -> bool {
    let pipe_hang_up = pipe_exec(file, PipeFunc::Hangup(true));
    pipe_hang_up
}

/// 管道文件的 "写端是否处于悬挂状态" 操作，用于检查管道文件的读端是否已经被关闭，效果等同于 !RingBuffer::is_read_wait
fn pipe_write_is_hang_up(file: Arc<File>) -> bool {
    pipe_exec(file, PipeFunc::Hangup(false))
}

/// (待实现)用于移动管道文件的文件指针，目前仅返回错误
fn pipe_llseek(_file: Arc<File>, _whence: SeekFrom) -> StrResult<u64> {
    Err("ESPIPE")
}
