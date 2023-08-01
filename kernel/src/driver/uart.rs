use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::sync::Arc;

use kernel_sync::Mutex;
use uart::{Uart16550Raw, Uart8250Raw};

use crate::device::UartDevice;
use crate::interrupt::DeviceBase;
use crate::task::schedule::schedule;
use crate::task::{current_task, Task, TaskState, TASK_MANAGER};

pub trait LowUartDriver: Send + Sync {
    fn _init(&mut self);
    fn _put(&mut self, c: u8);
    fn _read(&mut self) -> Option<u8>;
}

impl LowUartDriver for Uart16550Raw {
    fn _init(&mut self) {
        self.init()
    }

    fn _put(&mut self, c: u8) {
        self.put(c)
    }

    fn _read(&mut self) -> Option<u8> {
        self.read()
    }
}

impl<const W: usize> LowUartDriver for Uart8250Raw<W> {
    fn _init(&mut self) {
        self.init()
    }

    fn _put(&mut self, c: u8) {
        self.put(c)
    }

    fn _read(&mut self) -> Option<u8> {
        self.read()
    }
}

pub struct Uart16550 {
    inner: Mutex<(Box<dyn LowUartDriver>, UartInner)>,
}

struct UartInner {
    rx_buf: VecDeque<u8>,
    wait_queue: VecDeque<Arc<Task>>,
}

impl Uart16550 {
    pub fn new(uart_raw: Box<dyn LowUartDriver>) -> Self {
        let mut uart_raw = uart_raw;
        uart_raw._init();
        let inner = UartInner {
            rx_buf: VecDeque::new(),
            wait_queue: VecDeque::new(),
        };
        Uart16550 {
            inner: Mutex::new((uart_raw, inner)),
        }
    }
}

impl UartDevice for Uart16550 {
    fn put(&self, c: u8) {
        let mut inner = self.inner.lock();
        inner.0._put(c);
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
            if c == b'\n' {
                self.put(b'\r');
            }
            self.put(c);
        }
    }

    fn have_data_to_get(&self) -> bool {
        !self.inner.lock().1.rx_buf.is_empty()
    }

    fn have_space_to_put(&self) -> bool {
        true
    }
}

impl DeviceBase for Uart16550 {
    fn hand_irq(&self) {
        let mut inner = self.inner.lock();
        loop {
            if let Some(c) = inner.0._read() {
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
