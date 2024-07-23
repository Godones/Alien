use alloc::{collections::VecDeque, sync::Arc};
use core::{fmt::Debug, sync::atomic::AtomicU32};

use bitflags::bitflags;
use constants::{io::SeekFrom, AlienError, AlienResult};
use ksync::Mutex;
use shim::KTask;
use vfscore::{dentry::VfsDentry, inode::VfsInode, utils::VfsFileStat};

use crate::kfile::File;

static EVENTFD_ID: AtomicU32 = AtomicU32::new(0);

bitflags! {
    pub struct EventFdFlags: u32{
        const EFD_SEMAPHORE = 1;
        const EFD_CLOEXEC = 2;
        const EFD_NONBLOCK = 4;
    }
}

#[derive(Debug)]
pub struct EventFd {
    count: u64,
    flags: EventFdFlags,
    #[allow(unused)]
    id: u32,
}

impl EventFd {
    pub fn new(count: u64, flags: EventFdFlags, id: u32) -> Self {
        EventFd { count, flags, id }
    }
}

pub struct EventFdInode {
    eventfd: Mutex<EventFd>,
    wait_queue: Mutex<VecDeque<Arc<dyn KTask>>>,
}

impl Debug for EventFdInode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("EventFdInode")
            .field("eventfd", &self.eventfd)
            .finish()
    }
}

impl EventFdInode {
    pub fn new(eventfd: EventFd) -> Self {
        EventFdInode {
            eventfd: Mutex::new(eventfd),
            wait_queue: Mutex::new(VecDeque::new()),
        }
    }
}

impl File for EventFdInode {
    fn read(&self, buf: &mut [u8]) -> AlienResult<usize> {
        if buf.len() < 8 {
            return Err(AlienError::EINVAL);
        }
        let mut val = loop {
            let val = self.eventfd.lock().count;
            if val != 0 {
                break val;
            }
            if self
                .eventfd
                .lock()
                .flags
                .contains(EventFdFlags::EFD_NONBLOCK)
            {
                return Err(AlienError::EAGAIN);
            }
            let task = shim::take_current_task().unwrap();
            task.to_wait();
            self.wait_queue.lock().push_back(task.clone());
            shim::schedule_now(task); // yield current task
        };
        let mut eventfd = self.eventfd.lock();
        if eventfd.flags.contains(EventFdFlags::EFD_SEMAPHORE) {
            eventfd.count -= 1;
            val = 1;
        } else {
            eventfd.count = 0;
        }
        let val_bytes = val.to_ne_bytes();
        buf[..8].copy_from_slice(&val_bytes);
        return Ok(8);
    }
    fn write(&self, buf: &[u8]) -> AlienResult<usize> {
        if buf.len() < 8 {
            return Err(AlienError::EINVAL);
        }
        let val = u64::from_ne_bytes(buf[..8].try_into().unwrap());
        if val == u64::MAX {
            return Err(AlienError::EINVAL);
        }
        loop {
            let eventfd = self.eventfd.lock();
            if u64::MAX - eventfd.count > val {
                break;
            }
            // block until a read() is performed  on the
            // file descriptor, or fails with the error EAGAIN if the
            // file descriptor has been made nonblocking.
            if eventfd.flags.contains(EventFdFlags::EFD_NONBLOCK) {
                return Err(AlienError::EAGAIN);
            }
            drop(eventfd);
            // self.wait_queue.sleep();
            let task = shim::take_current_task().unwrap();
            task.to_wait();
            self.wait_queue.lock().push_back(task.clone());
            shim::schedule_now(task); // yield current task
        }
        let mut eventfd = self.eventfd.lock();
        eventfd.count += val;
        while let Some(task) = self.wait_queue.lock().pop_front() {
            task.to_wakeup();
            shim::put_task(task);
        }
        return Ok(8);
    }
    fn read_at(&self, _offset: u64, buf: &mut [u8]) -> AlienResult<usize> {
        self.read(buf)
    }
    fn write_at(&self, _offset: u64, _buf: &[u8]) -> AlienResult<usize> {
        self.write(_buf)
    }

    fn seek(&self, _pos: SeekFrom) -> AlienResult<u64> {
        Err(AlienError::ENOSYS)
    }

    fn get_attr(&self) -> AlienResult<VfsFileStat> {
        Err(AlienError::ENOSYS)
    }

    fn dentry(&self) -> Arc<dyn VfsDentry> {
        panic!("EventFdInode::dentry() is not implemented")
    }

    fn inode(&self) -> Arc<dyn VfsInode> {
        panic!("EventFdInode::inode() is not implemented")
    }

    fn is_readable(&self) -> bool {
        true
    }

    fn is_writable(&self) -> bool {
        true
    }

    fn is_append(&self) -> bool {
        true
    }
}

pub fn eventfd(init_val: u32, flags: u32) -> AlienResult<Arc<dyn File>> {
    let flags = EventFdFlags::from_bits(flags).ok_or(AlienError::EINVAL)?;
    let id = EVENTFD_ID.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    let eventfd = EventFd::new(init_val as u64, flags, id);
    let inode = Arc::new(EventFdInode::new(eventfd));
    Ok(inode)
}
