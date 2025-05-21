mod ebpf;
mod kprobe;

use alloc::{boxed::Box, sync::Arc};
use core::{any::Any, ffi::c_void, fmt::Debug};

use bpf_basic::{
    linux_bpf::{perf_event_attr, perf_type_id},
    perf::{bpf::BpfPerfEvent, PerfEventIoc, PerfProbeArgs},
};
use constants::{
    io::{PollEvents, SeekFrom},
    AlienError, AlienResult, LinuxErrno,
};
use kprobe::KprobePerfEvent;
use ksync::{Mutex, MutexGuard};
use vfs::kfile::File;
use vfscore::{dentry::VfsDentry, inode::VfsInode, utils::VfsFileStat};

use crate::{ebpf::EbpfKernelAuxiliary, task::current_task};

pub trait PerfEventOps: Send + Sync + Debug + Any {
    fn enable(&mut self) -> AlienResult<()>;
    fn disable(&mut self) -> AlienResult<()>;
    fn readable(&self) -> bool {
        false
    }
    fn writeable(&self) -> bool {
        false
    }
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[derive(Debug)]
pub struct PerfEvent {
    event: Mutex<Box<dyn PerfEventOps>>,
}

impl PerfEvent {
    pub fn new(event: Box<dyn PerfEventOps>) -> Self {
        PerfEvent {
            event: Mutex::new(event),
        }
    }

    pub fn event(&self) -> MutexGuard<Box<dyn PerfEventOps>> {
        self.event.lock()
    }
}

impl File for PerfEvent {
    fn read(&self, _buf: &mut [u8]) -> AlienResult<usize> {
        todo!()
    }
    fn write(&self, _buf: &[u8]) -> AlienResult<usize> {
        todo!()
    }

    fn seek(&self, _pos: SeekFrom) -> AlienResult<u64> {
        todo!()
    }

    fn get_attr(&self) -> AlienResult<VfsFileStat> {
        todo!()
    }

    fn ioctl(&self, cmd: u32, _arg: usize) -> AlienResult<usize> {
        let req = PerfEventIoc::try_from(cmd).map_err(|_| AlienError::EINVAL)?;
        log::error!("perf_event_ioctl: request: {:?}, arg: {}", req, _arg);
        match req {
            PerfEventIoc::Enable => {
                self.event.lock().enable().unwrap();
            }
            PerfEventIoc::Disable => {
                self.event.lock().disable().unwrap();
            }
            PerfEventIoc::SetBpf => {
                log::error!("perf_event_ioctl: PERF_EVENT_IOC_SET_BPF, arg: {}", _arg);
                let bpf_prog_fd = _arg;
                let task = current_task().unwrap();
                let file = task.get_file(bpf_prog_fd as _).ok_or(AlienError::EBADF)?;
                let mut event = self.event.lock();
                let kprobe_event = event
                    .as_any_mut()
                    .downcast_mut::<KprobePerfEvent>()
                    .ok_or(AlienError::EINVAL)?;
                kprobe_event.set_bpf_prog(file).unwrap();
            }
        }
        Ok(0)
    }

    fn dentry(&self) -> Arc<dyn VfsDentry> {
        todo!()
    }

    fn inode(&self) -> Arc<dyn VfsInode> {
        todo!()
    }

    fn is_readable(&self) -> bool {
        todo!()
    }

    fn is_writable(&self) -> bool {
        todo!()
    }

    fn is_append(&self) -> bool {
        false
    }

    fn poll(&self, _event: PollEvents) -> AlienResult<PollEvents> {
        let event = self.event.lock();
        let mut pollevent = PollEvents::empty();
        if event.readable() {
            pollevent.insert(PollEvents::EPOLLIN | PollEvents::EPOLLRDNORM);
        }
        if event.writeable() {
            pollevent.insert(PollEvents::EPOLLOUT);
        }
        // log::error!("perf_event_ioctl: got a poll event: {:?}", pollevent);
        Ok(pollevent)
    }
    fn mmap(&self, addr: usize, len: usize, offset: usize) -> AlienResult<()> {
        let mut event = self.event.lock();
        let event = event.as_any_mut().downcast_mut::<BpfPerfEvent>().unwrap();
        event.do_mmap(addr, len, offset).unwrap();
        Ok(())
    }
}

#[syscall_func(241)]
pub fn sys_perf_event_open(
    attr: *const u8,
    pid: i32,
    cpu: i32,
    group_fd: i32,
    flags: u32,
) -> AlienResult<isize> {
    let task = current_task().unwrap();
    let attr = task.transfer_raw_ptr(attr as *mut perf_event_attr);
    perf_event_open(attr, pid, cpu, group_fd, flags)
}

pub fn perf_event_open(
    attr: &perf_event_attr,
    pid: i32,
    cpu: i32,
    group_fd: i32,
    flags: u32,
) -> AlienResult<isize> {
    let args =
        PerfProbeArgs::try_from_perf_attr::<EbpfKernelAuxiliary>(attr, pid, cpu, group_fd, flags)
            .unwrap();
    println_color!(32, "perf_event_process: {:#?}", args);
    let event: Box<dyn PerfEventOps> = match args.type_ {
        // Kprobe
        // See /sys/bus/event_source/devices/kprobe/type
        perf_type_id::PERF_TYPE_MAX => {
            let kprobe_event = kprobe::perf_event_open_kprobe(args);
            Box::new(kprobe_event)
        }
        perf_type_id::PERF_TYPE_SOFTWARE => {
            let bpf_event = ebpf::perf_event_open_bpf(args);
            Box::new(bpf_event)
        }
        _ => {
            unimplemented!("perf_event_process: unknown type: {:?}", args);
        }
    };
    let event = Arc::new(PerfEvent::new(event));
    let task = current_task().unwrap();
    let fd = task.add_file(event).map_err(|_| LinuxErrno::EMFILE)?;
    Ok(fd as _)
}

pub fn perf_event_output(
    _ctx: *mut c_void,
    fd: usize,
    _flags: u32,
    data: &[u8],
) -> AlienResult<()> {
    // log::error!("perf_event_output: fd: {}, data: {:?}", fd, data.len());
    let task = current_task().unwrap();
    let file = task.get_file(fd).ok_or(AlienError::EBADF)?;
    let bpf_event_file = file
        .downcast_arc::<PerfEvent>()
        .map_err(|_| AlienError::EINVAL)?;
    let mut event = bpf_event_file.event();
    let event = event
        .as_any_mut()
        .downcast_mut::<BpfPerfEvent>()
        .ok_or(AlienError::EINVAL)?;
    event.write_event(data).unwrap();
    Ok(())
}
