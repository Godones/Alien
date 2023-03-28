// uart.rs
// UART routines and driver

use crate::config::RISCV_UART_ADDR;
use crate::driver::DeviceBase;
use crate::task::schedule::schedule;
use crate::task::{current_process, Process, ProcessState, PROCESS_MANAGER};
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::Error;
use core::fmt::Write;
use lazy_static::lazy_static;
use spin::{Mutex, Once};

pub trait CharDevice {
    fn put(&self, c: u8);
    fn get(&self) -> Option<u8>;
}

lazy_static! {
    pub static ref UART:Mutex<Ns16550a> = {
        let uart = Mutex::new(Ns16550a::new(RISCV_UART_ADDR));
        // uart.lock().init();
        uart
    };
}

lazy_static! {
    pub static ref USER_UART: Once<Arc<Uart>> = Once::new();
}

pub fn init_uart(base: usize) -> Arc<dyn DeviceBase> {
    let uart = Uart::new(base);
    uart.init();
    let uart = Arc::new(uart);
    USER_UART.call_once(|| uart.clone());
    uart
}

const READ_BUFFER_SIZE: usize = 128;

pub struct Ns16550a {
    base: usize,
}

pub struct Uart {
    inner: Mutex<UartInner>,
}
struct UartInner {
    device: Ns16550a,
    buffer: [u8; READ_BUFFER_SIZE],
    read_pos: usize,
    write_pos: usize,
    wait_queue: Vec<Arc<Process>>,
}

impl Ns16550a {
    pub const fn new(base: usize) -> Self {
        Self { base }
    }
    pub fn init(&self) {
        let ptr = self.base as *mut u8;
        unsafe {
            //disable interrupts
            ptr.add(1).write_volatile(0);
            // special mode to set baud rate.
            ptr.add(3).write_volatile(0x80);
            // LSB for baud rate of 38.4K
            ptr.add(0).write_volatile(0x03);
            // MSB for baud rate of 38.4k
            ptr.add(1).write_volatile(0x00);
            // set the world length to 8 bits
            ptr.add(3).write_volatile(3);
            // reset and enable FIFOs.
            ptr.add(2).write_volatile(0x7);
            // enable receive interrupts
            ptr.add(1).write_volatile(0x1);
        }
    }
}

impl Write for Ns16550a {
    fn write_str(&mut self, out: &str) -> Result<(), Error> {
        for c in out.bytes() {
            self.put(c);
        }
        Ok(())
    }
}

impl CharDevice for Ns16550a {
    fn put(&self, c: u8) {
        let ptr = self.base as *mut u8;
        loop {
            unsafe {
                let c = ptr.add(5).read_volatile();
                if c & (1 << 5) != 0 {
                    break;
                }
            }
        }
        unsafe {
            ptr.add(0).write_volatile(c);
        }
    }

    fn get(&self) -> Option<u8> {
        let ptr = self.base as *mut u8;
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

impl Uart {
    pub const fn new(base: usize) -> Self {
        Self {
            inner: Mutex::new(UartInner {
                device: Ns16550a::new(base),
                buffer: [0; READ_BUFFER_SIZE],
                read_pos: 0,
                write_pos: 0,
                wait_queue: vec![],
            }),
        }
    }
    pub fn init(&self) {
        self.inner.lock().device.init();
    }
}

impl CharDevice for Uart {
    fn put(&self, c: u8) {
        self.inner.lock().device.put(c);
    }
    fn get(&self) -> Option<u8> {
        loop {
            let mut inner = self.inner.lock();
            if inner.read_pos == inner.write_pos {
                let task = current_process().unwrap();
                inner.wait_queue.push(task.clone());
                task.update_state(ProcessState::Waiting);
                drop(inner);
                schedule();
            } else {
                let c = inner.buffer[inner.read_pos];
                inner.read_pos = (inner.read_pos + 1) % READ_BUFFER_SIZE;
                return Some(c);
            }
        }
    }
}

impl DeviceBase for Uart {
    fn hand_irq(&self) {
        let mut inner = self.inner.lock();
        while let Some(c) = inner.device.get() {
            let index = inner.write_pos;
            inner.buffer[index] = c;
            inner.write_pos = (inner.write_pos + 1) % READ_BUFFER_SIZE;
        }
        for task in inner.wait_queue.drain(..) {
            task.update_state(ProcessState::Ready);
            PROCESS_MANAGER.lock().push_back(task)
        }
    }
}
