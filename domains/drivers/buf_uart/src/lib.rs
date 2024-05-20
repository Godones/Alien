#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::{boxed::Box, collections::VecDeque, sync::Arc};
use core::fmt::Debug;

use basic::{println, sync::Mutex, AlienError, AlienResult};
use interface::{Basic, BufUartDomain, DeviceBase, DomainType, SchedulerDomain, UartDomain};
use spin::Once;

static UART: Once<Arc<dyn UartDomain>> = Once::new();
static SCHEDULER_DOMAIN: Once<Arc<dyn SchedulerDomain>> = Once::new();
#[derive(Debug)]
pub struct Uart {
    inner: Mutex<UartInner>,
}

#[derive(Debug)]
struct UartInner {
    rx_buf: VecDeque<u8>,
    wait_queue: VecDeque<usize>,
}

impl Uart {
    pub fn new() -> Self {
        let inner = UartInner {
            rx_buf: VecDeque::new(),
            wait_queue: VecDeque::new(),
        };
        Uart {
            inner: Mutex::new(inner),
        }
    }
}

impl Basic for Uart {}

impl DeviceBase for Uart {
    fn handle_irq(&self) -> AlienResult<()> {
        let mut inner = self.inner.lock();
        let uart = UART.get().unwrap();
        loop {
            if let Ok(Some(c)) = uart.getc() {
                inner.rx_buf.push_back(c);
                if !inner.wait_queue.is_empty() {
                    let tid = inner.wait_queue.pop_front().unwrap();
                    SCHEDULER_DOMAIN
                        .get()
                        .unwrap()
                        .wake_up_wait_task(tid)
                        .unwrap();
                }
            } else {
                break;
            }
        }
        Ok(())
    }
}

impl BufUartDomain for Uart {
    fn init(&self, uart_domain_name: &str) -> AlienResult<()> {
        let uart_domain = basic::get_domain(uart_domain_name).unwrap();
        match uart_domain {
            DomainType::UartDomain(uart) => {
                // enable receive interrupt
                // todo!(update it)
                uart.enable_receive_interrupt()?;
                UART.call_once(|| uart);
                Ok(())
            }
            ty => {
                println!("uart_domain_name: {},ty: {:?}", uart_domain_name, ty);
                Err(AlienError::EINVAL)
            }
        }?;
        let scheduler_domain = basic::get_domain("scheduler").unwrap();
        match scheduler_domain {
            DomainType::SchedulerDomain(scheduler_domain) => {
                SCHEDULER_DOMAIN.call_once(|| scheduler_domain);
                Ok(())
            }
            _ => return Err(AlienError::EINVAL),
        }
    }

    fn putc(&self, ch: u8) -> AlienResult<()> {
        let uart = UART.get().unwrap();
        if ch == b'\n' {
            uart.putc(b'\r')?;
        }
        uart.putc(ch)
    }

    fn getc(&self) -> AlienResult<Option<u8>> {
        loop {
            let mut inner = self.inner.lock();
            if inner.rx_buf.is_empty() {
                let scheduler = SCHEDULER_DOMAIN.get().unwrap();
                let tid = scheduler.current_tid()?.unwrap();
                inner.wait_queue.push_back(tid);
                drop(inner);
                scheduler.current_to_wait()?;
            } else {
                return Ok(inner.rx_buf.pop_front());
            }
        }
    }

    fn have_data_to_get(&self) -> AlienResult<bool> {
        Ok(!self.inner.lock().rx_buf.is_empty())
    }

    fn have_space_to_put(&self) -> AlienResult<bool> {
        Ok(true)
    }
}

pub fn main() -> Box<dyn BufUartDomain> {
    let uart = Uart::new();
    Box::new(uart)
}
