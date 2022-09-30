// uart.rs
// UART routines and driver

use core::fmt::Error;
use core::fmt::Write;
use crate::config::RISCV_UART_ADDR;
use super::Uart;

pub struct Ns16550a;

impl Write for Ns16550a {
    fn write_str(&mut self, out: &str) -> Result<(), Error> {
        for c in out.bytes() {
            self.put(c);
        }
        Ok(())
    }
}

impl Uart for Ns16550a {
    fn put(&mut self, c: u8) {
        let mut ptr = RISCV_UART_ADDR as *mut u8;
        loop {
            unsafe {
                let c = ptr.add(5).read_volatile();
                if c & (1 << 5) != 0 {
                    break;
                }
            }
        }
        ptr = RISCV_UART_ADDR as *mut u8;
        unsafe {
            ptr.add(0).write_volatile(c);
        }
    }

    fn get(&mut self) -> Option<u8> {
        let ptr = RISCV_UART_ADDR as *mut u8;
        unsafe {
            if ptr.add(5).read_volatile() & 1 == 0 {
                // The DR bit is 0, meaning no data
                None
            } else {
                // The DR bit is 1, meaning data!
                Some(ptr.add(0).read_volatile())
            }
        }
    }
}
