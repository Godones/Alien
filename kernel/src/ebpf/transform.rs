use alloc::{string::String, sync::Arc};
use core::ffi::c_void;

use arch::hart_id;
use bpf_basic::{map::UnifiedMap, BpfError, KernelAuxiliaryOps};
use constants::{AlienError, AlienResult};

use crate::{ebpf::map::BpfMap, perf::perf_event_output, task::current_task, time::read_time_ns};

pub fn bpf_error_to_alien(err: BpfError) -> AlienResult<isize> {
    match err {
        BpfError::InvalidArgument => Err(AlienError::EINVAL),
        BpfError::NotFound => Err(AlienError::EINVAL),
        BpfError::NotSupported => Err(AlienError::ENOSYS),
        BpfError::NoSpace => Err(AlienError::ENOSPC),
    }
}

pub fn bpf_error_to_err(err: BpfError) -> AlienError {
    match err {
        BpfError::InvalidArgument => AlienError::EINVAL,
        BpfError::NotFound => AlienError::EINVAL,
        BpfError::NotSupported => AlienError::ENOSYS,
        BpfError::NoSpace => AlienError::ENOSPC,
    }
}

#[derive(Debug)]
pub struct EbpfKernelAuxiliary;
impl KernelAuxiliaryOps for EbpfKernelAuxiliary {
    fn get_unified_map_from_ptr<F, R>(ptr: *const u8, func: F) -> bpf_basic::Result<R>
    where
        F: FnOnce(&mut UnifiedMap) -> bpf_basic::Result<R>,
    {
        let map = unsafe { Arc::from_raw(ptr as *const BpfMap) };
        let mut unified_map = map.unified_map();
        let ret = func(&mut *unified_map);
        drop(unified_map);
        // avoid double free
        // log::error!("get_unified_map_from_ptr: ret: {:?}", ret);
        let _ = Arc::into_raw(map);
        ret
    }

    fn get_unified_map_from_fd<F, R>(map_fd: u32, func: F) -> bpf_basic::Result<R>
    where
        F: FnOnce(&mut UnifiedMap) -> bpf_basic::Result<R>,
    {
        // log::error!("get_unified_map_from_fd: map_fd: {}", map_fd);
        let task = current_task().unwrap();
        let file = task.get_file(map_fd as _).ok_or(BpfError::NotFound)?;
        let bpf_map = file.as_any().downcast_ref::<BpfMap>().unwrap();
        let unified_map = &mut bpf_map.unified_map();
        func(unified_map)
    }

    fn get_unified_map_ptr_from_fd(map_fd: u32) -> bpf_basic::Result<*const u8> {
        // log::error!("get_unified_map_ptr_from_fd: map_fd: {}", map_fd);
        let task = current_task().unwrap();
        let file = task.get_file(map_fd as _).ok_or(BpfError::NotFound)?;
        let bpf_map = file
            .downcast_arc::<BpfMap>()
            .map_err(|_| BpfError::NotFound)?;
        let map_ptr = Arc::into_raw(bpf_map) as usize;
        Ok(map_ptr as *const u8)
    }

    fn transmute_buf(ptr: *const u8, size: usize) -> bpf_basic::Result<&'static [u8]> {
        let task = current_task().unwrap();
        let buf = task.transfer_buffer(ptr, size);
        assert_eq!(buf.len(), 1);
        let buf = buf.into_iter().next().unwrap();
        Ok(buf)
    }

    fn transmute_buf_mut(ptr: *mut u8, size: usize) -> bpf_basic::Result<&'static mut [u8]> {
        let task = current_task().unwrap();
        let buf = task.transfer_buffer(ptr, size);
        assert_eq!(buf.len(), 1);
        let buf = buf.into_iter().next().unwrap();
        Ok(buf)
    }

    fn current_cpu_id() -> u32 {
        hart_id() as _
    }

    fn perf_event_output(
        ctx: *mut c_void,
        fd: u32,
        flags: u32,
        data: &[u8],
    ) -> bpf_basic::Result<()> {
        perf_event_output(ctx, fd as usize, flags, data).map_err(|_| BpfError::NotSupported)?;
        Ok(())
    }

    fn string_from_user_cstr(ptr: *const u8) -> bpf_basic::Result<String> {
        let task = current_task().unwrap();
        let string = task.transfer_str(ptr);
        log::error!("string_from_user_cstr: string: {:?}", string);
        Ok(string)
    }

    fn ebpf_write_str(str: &str) -> bpf_basic::Result<()> {
        println!("ebpf_write_str: str: {:?}", str);
        Ok(())
    }

    fn ebpf_time_ns() -> bpf_basic::Result<u64> {
        let now = read_time_ns();
        Ok(now)
    }
}
