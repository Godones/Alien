use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use constants::{
    io::{OpenFlags, PollEvents, SeekFrom},
    time::{ClockId, ITimeSpec, TimeSpec},
    AlienError, AlienResult,
};
use ksync::Mutex;
use timer::{TimeNow, ToClock};
use vfscore::{dentry::VfsDentry, inode::VfsInode, utils::VfsFileStat};

use crate::kfile::File;

#[derive(Debug)]
pub struct TimerFile {
    flags: OpenFlags,
    timer: Mutex<ITimeSpec>,
    timer_next_clock: AtomicUsize,
    timer_interval_clock: AtomicUsize,
    /// Record the number of ticks that have been triggered
    ticks: AtomicUsize,
    disable: AtomicBool,
    #[allow(unused)]
    id: ClockId,
}

impl TimerFile {
    pub fn new(flags: OpenFlags, timer: ITimeSpec, id: ClockId) -> Self {
        TimerFile {
            flags,
            timer: Mutex::new(timer),
            ticks: AtomicUsize::new(0),
            timer_interval_clock: AtomicUsize::new(0),
            timer_next_clock: AtomicUsize::new(0),
            disable: AtomicBool::new(true),
            id,
        }
    }

    /// Return the interval of the timer
    pub fn get_interval(&self) -> TimeSpec {
        self.timer.lock().it_interval
    }

    /// Return the next expiration time
    pub fn get_it_value(&self) -> TimeSpec {
        self.timer.lock().it_value
    }

    /// Reset the timer
    pub fn set_timer(&self, timer: ITimeSpec) {
        if timer.it_value == TimeSpec::default() {
            self.disable.store(true, Ordering::Relaxed);
        } else {
            self.disable.store(false, Ordering::Relaxed);
        }
        let next_clock = timer.it_value.to_clock() + TimeSpec::now().to_clock();
        let interval_clock = timer.it_interval.to_clock();
        *self.timer.lock() = timer;
        self.timer_next_clock.store(next_clock, Ordering::Relaxed);
        self.timer_interval_clock
            .store(interval_clock, Ordering::Relaxed);
    }

    pub fn calculate_ticks(&self) {
        if self.disable.load(Ordering::Relaxed) {
            return;
        }
        let now = TimeSpec::now().to_clock();
        let mut t_ticks = 0;
        let next_clock = self.timer_next_clock.load(Ordering::Relaxed);
        let interval_clock = self.timer_interval_clock.load(Ordering::Relaxed);
        if now > next_clock {
            t_ticks += 1;
            if interval_clock != 0 {
                let diff = now - next_clock;
                let nums = diff / interval_clock;
                t_ticks += nums;
            }
            // update next_clock
            let next_clock = now + interval_clock;
            self.timer_next_clock.store(next_clock, Ordering::Relaxed);
            self.ticks.fetch_add(t_ticks, Ordering::Relaxed);
        }
    }
}

impl File for TimerFile {
    fn read(&self, buf: &mut [u8]) -> AlienResult<usize> {
        if buf.len() != 8 {
            return Err(AlienError::EINVAL);
        }
        let ticks = loop {
            self.calculate_ticks();
            let ticks = self.ticks.load(Ordering::Relaxed);
            if ticks != 0 {
                // reset ticks
                self.ticks.store(0, Ordering::Relaxed);
                break ticks;
            }
            if self.flags.contains(OpenFlags::O_NONBLOCK) {
                return Err(AlienError::EAGAIN);
            } else {
                shim::suspend();
            }
        };
        let bytes = ticks.to_ne_bytes();
        buf.copy_from_slice(&bytes);
        Ok(8)
    }

    fn write(&self, _buf: &[u8]) -> AlienResult<usize> {
        Err(AlienError::EINVAL)
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
        panic!("TimerFile does not have attr")
    }

    fn ioctl(&self, _cmd: u32, _arg: usize) -> AlienResult<usize> {
        panic!("ioctl is not implemented for TimerFile")
    }

    fn dentry(&self) -> Arc<dyn VfsDentry> {
        panic!("TimerFile does not have dentry")
    }

    fn inode(&self) -> Arc<dyn VfsInode> {
        panic!("TimerFile does not have inode")
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
    fn poll(&self, event: PollEvents) -> AlienResult<PollEvents> {
        if self.ticks.load(Ordering::Relaxed) != 0 && event.contains(PollEvents::EPOLLIN) {
            return Ok(PollEvents::EPOLLIN);
        }
        Ok(PollEvents::empty())
    }
}
