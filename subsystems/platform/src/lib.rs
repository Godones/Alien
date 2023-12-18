#![no_std]
#![feature(naked_functions)]
#[macro_use]
pub mod console;
mod logger;
#[cfg(feature = "qemu_riscv")]
mod qemu_riscv;
#[cfg(feature = "startfive2_riscv")]
mod startfive2_riscv;

#[cfg(feature = "qemu_riscv")]
use qemu_riscv::console_putchar;

#[cfg(feature = "startfive2_riscv")]
use startfive2_riscv::console_putchar;

#[cfg(feature = "qemu_riscv")]
pub use qemu_riscv::system_shutdown;

#[cfg(feature = "startfive2_riscv")]
pub use startfive2_riscv::system_shutdown;

pub fn platform_init() {
    logger::init_logger();
}
