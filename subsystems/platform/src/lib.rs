#![no_std]
#![feature(naked_functions)]
#![feature(asm_const)]
extern crate alloc;

#[macro_use]
pub mod console;
mod common_riscv;
#[cfg(feature = "hifive")]
mod hifive_riscv;


use spin::Once;
pub use basemachine::MachineInfo as PlatformInfo;
use ::config::CPU_NUM;

#[cfg(feature = "qemu_riscv")]
mod qemu_riscv;
#[cfg(feature = "vf2")]
mod starfive2_riscv;
pub mod logging;

#[cfg(feature = "qemu_riscv")]
use qemu_riscv::console_putchar;
#[cfg(feature = "qemu_riscv")]
pub use qemu_riscv::{config, set_timer, system_shutdown};

#[cfg(feature = "vf2")]
use starfive2_riscv::console_putchar;
#[cfg(feature = "vf2")]
pub use starfive2_riscv::{config, set_timer, system_shutdown};

#[cfg(feature = "hifive")]
use hifive_riscv::console_putchar;

#[cfg(feature = "hifive")]
pub use hifive_riscv::{config, set_timer, system_shutdown};
use crate::common_riscv::sbi::hart_start;
use crate::console::PrePrint;



#[no_mangle]
pub fn platform_init(hart_id:usize,dtb: usize) {
    println!("{}",::config::FLAG);
    #[cfg(feature = "hifive")]
    hifive_riscv::init_dtb(Some(dtb));
    #[cfg(feature = "vf2")]
    starfive2_riscv::init_dtb(Some(dtb));
    #[cfg(feature = "qemu_riscv")]
    qemu_riscv::init_dtb(Some(dtb));
    let machine_info = basemachine::machine_info_from_dtb(platform_dtb_ptr());
    MACHINE_INFO.call_once(|| machine_info);
    logging::init_logger();
    preprint::init_print(&PrePrint);
    #[cfg(feature = "smp")]
    init_other_hart(hart_id);
    unsafe{
        main(hart_id)
    }
}


/// 唤醒其它核
///
/// 对于qemu来说，只需要工具所有的核都是一样的，因此从严号核开始唤醒。
/// 对于visionfive2/unmatched 来说，0号核只有M态，因此不进行唤醒
fn init_other_hart(hart_id: usize) {
    let start_hart = if cfg!(any(feature = "vf2", feature = "hifive")){
        1
    } else {
        0
    };
    for i in start_hart..CPU_NUM {
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
    #[cfg(feature = "hifive")]
    return *hifive_riscv::DTB.get().unwrap();
    #[cfg(feature = "vf2")]
    return *starfive2_riscv::DTB.get().unwrap();
    #[cfg(feature = "qemu_riscv")]
    return *qemu_riscv::DTB.get().unwrap();
}

static MACHINE_INFO: Once<PlatformInfo> = Once::new();

pub fn platform_machine_info()->PlatformInfo{
    MACHINE_INFO.get().unwrap().clone()
}
