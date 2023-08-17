//! UART driver
//!
//! This module provides a driver for UARTs. It support uart8250 and uart16550.
//! For uart8250,it support 1byte/4bytes width register.
//!
//! # Example
//!```
//! use uart::Uart8250Raw;
//! use uart::Uart16550Raw;
//! let uart = Uart8250Raw::<4>::new(0x10000);
//! uart.init();
//! let uart = Uart16550Raw::new(0x10000);
//! uart.init();
//! uart.put(b'a');
//! uart.read();
//! ```
//!
#![no_std]

pub use uart16550::Uart16550Raw;
pub use uart8250::Uart8250Raw;

mod uart16550;
mod uart8250;
