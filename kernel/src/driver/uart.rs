use alloc::collections::VecDeque;
use alloc::sync::Arc;
use core::sync::atomic::Ordering;

use lazy_static::lazy_static;
use spin::Once;

use kernel_sync::Mutex;

use crate::driver::DeviceBase;
use crate::print::console::UART_FLAG;
use crate::task::schedule::schedule;
use crate::task::{current_task, Task, TaskState, TASK_MANAGER};

pub trait CharDevice {
    fn put(&self, c: u8);
    fn get(&self) -> Option<u8>;
    fn put_bytes(&self, bytes: &[u8]);
}

lazy_static! {
    pub static ref USER_UART: Once<Arc<Uart>> = Once::new();
}

pub fn init_uart(base: usize) -> Arc<dyn DeviceBase> {
    let uart = Uart::new(base);
    let uart = Arc::new(uart);
    USER_UART.call_once(|| uart.clone());
    UART_FLAG.store(true, Ordering::Relaxed);
    uart
}

pub struct Uart {
    inner: Mutex<(UartRaw, UartInner)>,
}

struct UartInner {
    rx_buf: VecDeque<u8>,
    wait_queue: VecDeque<Arc<Task>>,
}

struct UartRaw(usize);

impl UartRaw {
    fn init(&self) {
        let ptr = self.0 as *mut u8;
        unsafe {
            //disable interrupts
            ptr.add(1).write_volatile(0);
            // // special mode to set baud rate.
            ptr.add(3).write_volatile(0x80);
            // // LSB for baud rate of 38.4K
            ptr.add(0).write_volatile(0x03);
            // // MSB for baud rate of 38.4k
            ptr.add(1).write_volatile(0x00);
            // // set the world length to 8 bits
            ptr.add(3).write_volatile(3);
            // reset and enable FIFOs.
            ptr.add(2).write_volatile(0x7);
            // enable receive interrupts and transmit interrupts
            ptr.add(1).write_volatile(0x3);
            // ptr.add(1).write_volatile(0x1);
        }
    }
    pub fn put(&self, c: u8) {
        let ptr = self.0 as *mut u8;
        unsafe {
            // wait for transmitter to be ready
            while ptr.add(5).read_volatile() & 0x20 == 0 {}
            // write
            ptr.add(0).write_volatile(c);
        }
    }
    pub fn read(&self) -> Option<u8> {
        let ptr = self.0 as *mut u8;
        unsafe {
            // check if there is data
            if ptr.add(5).read_volatile() & 1 == 0 {
                None
            } else {
                Some(ptr.add(0).read_volatile())
            }
        }
    }
}

impl Uart {
    pub fn new(base: usize) -> Self {
        let uart_raw = UartRaw(base);
        uart_raw.init();
        let inner = UartInner {
            rx_buf: VecDeque::new(),
            wait_queue: VecDeque::new(),
        };
        Uart {
            inner: Mutex::new((uart_raw, inner)),
        }
    }
}

impl CharDevice for Uart {
    fn put(&self, c: u8) {
        let inner = self.inner.lock();
        inner.0.put(c);
    }
    fn get(&self) -> Option<u8> {
        loop {
            let mut inner = self.inner.lock();
            if inner.1.rx_buf.is_empty() {
                let current_process = current_task().unwrap();
                current_process.update_state(TaskState::Waiting);
                inner.1.wait_queue.push_back(current_process.clone());
                drop(inner);
                schedule();
            } else {
                return inner.1.rx_buf.pop_front();
            }
        }
    }

    fn put_bytes(&self, bytes: &[u8]) {
        for &c in bytes {
            self.put(c);
        }
    }
}

impl DeviceBase for Uart {
    fn hand_irq(&self) {
        let mut inner = self.inner.lock();
        loop {
            if let Some(c) = inner.0.read() {
                inner.1.rx_buf.push_back(c);
                if !inner.1.wait_queue.is_empty() {
                    let process = inner.1.wait_queue.pop_front().unwrap();
                    process.update_state(TaskState::Ready);
                    let mut guard = TASK_MANAGER.lock();
                    guard.push_back(process);
                }
            } else {
                break;
            }
        }
    }
}
