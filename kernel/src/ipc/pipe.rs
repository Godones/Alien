use alloc::boxed::Box;
use alloc::sync::{Arc,Weak};
use core::intrinsics::forget;
use rvfs::dentry::DirEntry;
use rvfs::file::{File, FileMode, FileOps, OpenFlags};
use rvfs::inode::{SpecialData};
use rvfs::mount::VfsMount;
use rvfs::StrResult;
use crate::task::do_suspend;


const PIPE_BUF: usize = 512;
pub struct Pipe;

pub struct RingBuffer{
    pub buf: [u8;PIPE_BUF],
    pub head: usize,
    pub tail: usize,
    pub read_wait: Option<Weak<File>>, // record whether there is a process waiting for reading
    pub write_wait: Option<Weak<File>>,// record whether there is a process waiting for writing
    pub ref_count: usize,
}

impl RingBuffer {
    pub fn new()->RingBuffer{
        RingBuffer{
            buf: [0;PIPE_BUF],
            head: 0,
            tail: 0,
            read_wait: None,
            write_wait: None,
            ref_count: 2,
        }
    }
    pub fn is_empty(&self)->bool{
        self.head == self.tail
    }
    pub fn is_full(&self)->bool{
        (self.tail + 1) % PIPE_BUF == self.head
    }
    /// return the number of bytes that can
    pub fn available_read(&self)->usize{
        if self.head <= self.tail{
            self.tail - self.head
        }else{
            PIPE_BUF - self.head + self.tail
        }
    }
    /// return the number of bytes that can be written
    pub fn available_write(&self)->usize{
        if self.head <= self.tail{
            PIPE_BUF - self.tail + self.head - 1
        }else{
            self.head - self.tail - 1
        }
    }
    pub fn write(&mut self, buf: &[u8])->usize{
        let mut count = 0;
        while !self.is_full() && count < buf.len(){
            self.buf[self.tail] = buf[count];
            self.tail = (self.tail + 1) % PIPE_BUF;
            count += 1;
        }
        count
    }
    pub fn read(&mut self, buf: &mut [u8])->usize{
        let mut count = 0;
        while !self.is_empty() && count < buf.len(){
            buf[count] = self.buf[self.head];
            self.head = (self.head + 1) % PIPE_BUF;
            count += 1;
        }
        count
    }
    pub fn clear(&mut self){
        self.head = 0;
        self.tail = 0;
    }
    pub fn is_write_wait(&self)->bool{
        self.write_wait.is_some() && self.write_wait.as_ref().unwrap().upgrade().is_some()
    }

    pub fn is_read_wait(&self)->bool{
        self.read_wait.is_some() && self.read_wait.as_ref().unwrap().upgrade().is_some()
    }
}


impl Pipe{
    pub fn new()->(Arc<File>,Arc<File>){
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
            ops
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
            ops
        };
        let (rx_file,tx_file) = (Arc::new(rx_file),Arc::new(tx_file));
        buf.read_wait = Some(Arc::downgrade(&rx_file));
        buf.write_wait = Some(Arc::downgrade(&tx_file));
        let ptr = Box::into_raw(buf) as *const u8;
        rx_file.f_dentry.access_inner().d_inode.access_inner().special_data = Some( SpecialData::PipeData(ptr));
        tx_file.f_dentry.access_inner().d_inode.access_inner().special_data = Some( SpecialData::PipeData(ptr));
        (rx_file,tx_file)
    }
}


fn pipe_write(file: Arc<File>, user_buf: &[u8], _offset: u64) -> StrResult<usize>{
    debug!("pipe_write: {:?}, mode:{:?}", user_buf.len(),file.f_mode);
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
        let mut buf = unsafe{Box::from_raw(ptr as *mut RingBuffer)};
        let available = buf.available_write();
        if available == 0{
            if !buf.is_read_wait(){
                // if there is no process waiting for reading, we should return
                forget(buf);
                break;
            }
            // wait for reading
            drop(inode_inner);
            do_suspend();
        }else {
            // let min = core::cmp::min(available, user_buf.len());
            let min = core::cmp::min(available, user_buf.len() - count);
            count += buf.write(&user_buf[count..count+min]);
            if count == user_buf.len(){
                forget(buf);
                break;
            }
        }
        forget(buf); // we can't drop the buf here, because the inode still holds the pointer
    }
    debug!("pipe_write: count:{}", count);
    Ok(count)
}


fn pipe_read(file: Arc<File>, user_buf: &mut [u8], _offset: u64) -> StrResult<usize>{
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
        let mut buf = unsafe{Box::from_raw(ptr as *mut RingBuffer)};
        let available = buf.available_read();
        // warn!("pipe_read: available:{}", available);
        if available == 0{
            if !buf.is_write_wait(){
                // if there is no process waiting for writing, we should return
                forget(buf);
                break
            }
            // wait for writing
            drop(inode_inner);
            do_suspend();
        }else {
            let min = core::cmp::min(available, user_buf.len() - count);
            count += buf.read(&mut user_buf[count..count+min]);
            if count == user_buf.len(){
                forget(buf);
                break;
            }
        }
        forget(buf); // we can't drop the buf here, because the inode still holds the pointer
    }
    debug!("pipe_read: return count:{}",count);
    Ok(count)
}

fn pipe_release(file: Arc<File>)->StrResult<()>{
    debug!("pipe_release: file");
    assert_eq!(Arc::strong_count(&file),1);
    let inode = &file.f_dentry.access_inner().d_inode;
    assert!(inode.access_inner().special_data.is_some());
    let inode_inner = inode.access_inner();
    let data = inode_inner.special_data.as_ref().unwrap();
    let ptr = match data {
        SpecialData::PipeData(ptr) => *ptr,
        _ => panic!("pipe_release: invalid special data"),
    };
    let mut buf = unsafe{Box::from_raw(ptr as *mut RingBuffer)};
    buf.ref_count -= 1;
    if buf.ref_count == 0{
        // the last pipe file is closed, we should free the buffer
        debug!("pipe_release: free buffer");
        drop(buf);
    }else {
        forget(buf)
    }
    Ok(())
}