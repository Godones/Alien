mod map;
pub mod prog;
mod transform;

use alloc::{collections::BTreeMap, vec};

use bpf_basic::{
    helper::RawBPFHelperFn,
    linux_bpf::{bpf_attr, bpf_cmd},
    map::{BpfMapGetNextKeyArg, BpfMapUpdateArg},
};
use constants::{AlienError, AlienResult};
use spin::Lazy;
pub use transform::EbpfKernelAuxiliary;

use crate::{ebpf::transform::bpf_error_to_alien, task::current_task};

pub static BPF_HELPER_FUN_SET: Lazy<BTreeMap<u32, RawBPFHelperFn>> =
    Lazy::new(bpf_basic::helper::init_helper_functions::<EbpfKernelAuxiliary>);

#[syscall_func(280)]
fn syscall_bpf(cmd: u32, attr: *mut u8, size: u32) -> AlienResult<isize> {
    let task = current_task().unwrap();
    let mut buf = vec![0u8; size as usize];
    task.access_inner()
        .copy_from_user_buffer(attr as *const u8, buf.as_mut_ptr(), size as usize);
    let attr = unsafe { &*(buf.as_ptr() as *const bpf_attr) };
    let cmd = bpf_cmd::try_from(cmd).map_err(|_| AlienError::EINVAL)?;
    bpf(cmd, &attr)
}

pub fn bpf(cmd: bpf_cmd, attr: &bpf_attr) -> AlienResult<isize> {
    println_color!(31, "bpf cmd: [{:?}]", cmd);
    let update_arg = BpfMapUpdateArg::try_from(attr).map_err(|_| AlienError::EINVAL)?;
    match cmd {
        // Map related commands
        bpf_cmd::BPF_MAP_CREATE => map::bpf_map_create(attr),
        bpf_cmd::BPF_MAP_UPDATE_ELEM => {
            bpf_basic::map::bpf_map_update_elem::<EbpfKernelAuxiliary>(update_arg)
                .map_or_else(bpf_error_to_alien, |_| Ok(0))
        }
        bpf_cmd::BPF_MAP_LOOKUP_ELEM => {
            bpf_basic::map::bpf_lookup_elem::<EbpfKernelAuxiliary>(update_arg)
                .map_or_else(bpf_error_to_alien, |_| Ok(0))
        }
        bpf_cmd::BPF_MAP_GET_NEXT_KEY => {
            let update_arg = BpfMapGetNextKeyArg::try_from(attr).map_err(|_| AlienError::EINVAL)?;
            bpf_basic::map::bpf_map_get_next_key::<EbpfKernelAuxiliary>(update_arg)
                .map_or_else(bpf_error_to_alien, |_| Ok(0))
        }
        bpf_cmd::BPF_MAP_DELETE_ELEM => {
            bpf_basic::map::bpf_map_delete_elem::<EbpfKernelAuxiliary>(update_arg)
                .map_or_else(bpf_error_to_alien, |_| Ok(0))
        }
        bpf_cmd::BPF_MAP_LOOKUP_AND_DELETE_ELEM => {
            bpf_basic::map::bpf_map_lookup_and_delete_elem::<EbpfKernelAuxiliary>(update_arg)
                .map_or_else(bpf_error_to_alien, |_| Ok(0))
        }
        bpf_cmd::BPF_MAP_LOOKUP_BATCH => {
            bpf_basic::map::bpf_map_lookup_batch::<EbpfKernelAuxiliary>(update_arg)
                .map_or_else(bpf_error_to_alien, |_| Ok(0))
        }
        bpf_cmd::BPF_MAP_FREEZE => {
            bpf_basic::map::bpf_map_freeze::<EbpfKernelAuxiliary>(update_arg)
                .map_or_else(bpf_error_to_alien, |_| Ok(0))
        }
        // Program related commands
        bpf_cmd::BPF_PROG_LOAD => prog::bpf_prog_load(attr),
        // Object creation commands
        bpf_cmd::BPF_BTF_LOAD | bpf_cmd::BPF_LINK_CREATE | bpf_cmd::BPF_OBJ_GET_INFO_BY_FD => {
            log::error!("bpf cmd: [{:?}] not implemented", cmd);
            Err(AlienError::ENOSYS)
        }
        ty => {
            unimplemented!("bpf cmd: [{:?}] not implemented", ty)
        }
    }
}
