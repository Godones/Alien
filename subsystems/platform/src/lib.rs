#![no_std]
#![feature(naked_functions)]

extern crate alloc;

#[macro_use]
pub mod console;
mod common_riscv;
#[cfg(feature = "hifive_riscv")]
mod hifive_riscv;
mod logger;
#[cfg(feature = "qemu_riscv")]
mod qemu_riscv;
#[cfg(feature = "starfive2_riscv")]
mod starfive2_riscv;

#[cfg(feature = "qemu_riscv")]
use qemu_riscv::console_putchar;
#[cfg(feature = "qemu_riscv")]
pub use qemu_riscv::system_shutdown;

#[cfg(feature = "starfive2_riscv")]
use starfive2_riscv::console_putchar;
#[cfg(feature = "starfive2_riscv")]
pub use starfive2_riscv::system_shutdown;

#[cfg(feature = "hifive_riscv")]
pub use hifive_riscv::system_shutdown;

#[cfg(feature = "hifive_riscv")]
pub use hifive_riscv::console_putchar;

pub fn platform_init(dtb: Option<usize>) {
    #[cfg(feature = "hifive_riscv")]
    hifive_riscv::init_dtb(dtb);
    #[cfg(feature = "starfive2_riscv")]
    starfive2_riscv::init_dtb(dtb);
    #[cfg(feature = "qemu_riscv")]
    qemu_riscv::init_dtb(dtb);
    logger::init_logger();
}

pub fn platform_dtb_ptr() -> usize {
    #[cfg(feature = "hifive_riscv")]
    return hifive_riscv::DTB.get().unwrap();
    #[cfg(feature = "starfive2_riscv")]
    return starfive2_riscv::DTB.get().unwrap();
    #[cfg(feature = "qemu_riscv")]
    return qemu_riscv::DTB.get().unwrap();
}
