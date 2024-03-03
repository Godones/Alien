#![no_std]
#![deny(unsafe_code)]
extern crate alloc;

use alloc::collections::VecDeque;
use alloc::sync::Arc;
use core::fmt::{Debug, Formatter};
use interface::{Basic, UartDomain};
use ksync::Mutex;
use libsyscall::KTask;
use rref::RpcResult;

pub struct Uart {
    inner: Mutex<(Arc<dyn UartDomain>, UartInner)>,
}

struct UartInner {
    rx_buf: VecDeque<u8>,
    wait_queue: VecDeque<Arc<dyn KTask>>,
}

impl Uart {
    pub fn new(uart_raw: Arc<dyn UartDomain>) -> Self {
        let mut uart_raw = uart_raw;
        // uart_raw._init();
        let inner = UartInner {
            rx_buf: VecDeque::new(),
            wait_queue: VecDeque::new(),
        };
        Uart {
            inner: Mutex::new((uart_raw, inner)),
        }
    }
}

impl Basic for Uart {}

impl Debug for Uart {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Uart").finish()
    }
}

impl UartDomain for Uart {
    fn putc(&self, ch: u8) -> RpcResult<()> {
        let mut inner = self.inner.lock();
        if ch == b'\n' {
            inner.0.putc(b'\r')
        } else {
            inner.0.putc(ch)
        }
    }

    fn getc(&self) -> RpcResult<Option<u8>> {
        loop {
            let mut inner = self.inner.lock();
            if inner.1.rx_buf.is_empty() {
                // let current_process = current_task().unwrap();
                // current_process.update_state(TaskState::Waiting);
                let task = libsyscall::current_task();
                task.to_wait();
                inner.1.wait_queue.push_back(task);
                drop(inner);
                libsyscall::suspend();
            } else {
                return Ok(inner.1.rx_buf.pop_front());
            }
        }
    }

    fn have_data_to_get(&self) -> bool {
        !self.inner.lock().1.rx_buf.is_empty()
    }

    fn have_space_to_put(&self) -> bool {
        true
    }

    fn handle_irq(&self) {
        loop {
            let mut inner = self.inner.lock();
            if let Some(c) = inner.0._read() {
                inner.1.rx_buf.push_back(c);
                if !inner.1.wait_queue.is_empty() {
                    let task = inner.1.wait_queue.pop_front().unwrap();
                    task.to_wakeup();
                    libsyscall::put_task(task);
                }
            } else {
                break;
            }
        }
    }
}

fn main() -> Arc<dyn UartDomain> {
    let low_level_uart = libsyscall::get_uart_domain().unwrap();
    let uart = Uart::new(low_level_uart);
    Arc::new(uart)
}
