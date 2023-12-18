#![no_std]

#[cfg(feature = "rv")]
mod riscv;

#[cfg(feature = "rv")]
pub use riscv::*;
