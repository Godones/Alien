#![no_std]

pub use uart16550::Uart16550Raw;
pub use uart8250::Uart8250Raw;

mod uart16550;
mod uart8250;
