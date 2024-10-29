#![no_std]
#![feature(naked_functions)]
#[macro_use]
pub mod console;
mod common_riscv;
mod logger;
#[cfg(qemu_riscv)]
mod qemu_riscv;

#[cfg(vf2)]
mod starfive2_riscv;

pub use common_riscv::{basic::MachineInfo as PlatformInfo, sbi::remote_fence_i};
#[cfg(qemu_riscv)]
use qemu_riscv::*;
#[cfg(qemu_riscv)]
pub use qemu_riscv::{config, set_timer, system_shutdown};
use spin::Once;
#[cfg(vf2)]
pub use starfive2_riscv::{config, set_timer, system_shutdown};

use crate::common_riscv::sbi::hart_start;
#[cfg(vf2)]
use crate::starfive2_riscv::*;

extern "C" {
    fn sbss();
    fn ebss();
}

/// 清空.bss段
fn clear_bss() {
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

pub fn platform_init(hart_id: usize, dtb: usize) {
    clear_bss();
    println!("{}", ::config::ALIEN_FLAG);
    init_dtb(Some(dtb));
    let machine_info = basic_machine_info();
    MACHINE_INFO.call_once(|| machine_info);
    logger::init_logger();
    init_other_hart(hart_id);
    unsafe { main(hart_id) }
}

fn init_other_hart(hart_id: usize) {
    let start_hart = if cfg!(vf2) { 1 } else { 0 };
    for i in start_hart..::config::CPU_NUM {
        if i != hart_id {
            let res = hart_start(i, _start_secondary as usize, 0);
            assert_eq!(res.error, 0);
        }
    }
}

extern "C" {
    fn main(hart_id: usize);
    fn _start_secondary();
}

pub fn platform_dtb_ptr() -> usize {
    #[cfg(vf2)]
    return *starfive2_riscv::DTB.get().unwrap();
    #[cfg(qemu_riscv)]
    return *qemu_riscv::DTB.get().unwrap();
}

static MACHINE_INFO: Once<PlatformInfo> = Once::new();

pub fn platform_machine_info() -> PlatformInfo {
    MACHINE_INFO.get().unwrap().clone()
}
