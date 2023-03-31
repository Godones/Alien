// uart.rs
// UART routines and driver

use crate::config::RISCV_UART_ADDR;
use crate::driver::DeviceBase;
use crate::task::schedule::schedule;
use crate::task::{current_process, Process, ProcessState, PROCESS_MANAGER};
use alloc::sync::Arc;
use lazy_static::lazy_static;
use spin::{Mutex, Once};
use uart::{Uart, UartRaw};

pub trait CharDevice {
    fn put(&self, c: u8);
    fn get(&self) -> Option<u8>;
    fn put_bytes(&self, bytes: &[u8]) ;
}

lazy_static! {
    pub static ref UART: Mutex<UartRaw> = {
        let uart = Mutex::new(UartRaw::new(RISCV_UART_ADDR));
        uart
    };
}

lazy_static! {
    pub static ref USER_UART: Once<Arc<UartWrapper>> = Once::new();
}

pub fn init_uart(base: usize) -> Arc<dyn DeviceBase> {
    let uart = UartWrapper::new(base);
    let uart = Arc::new(uart);
    USER_UART.call_once(|| uart.clone());
    uart
}

pub struct UartWrapper(Uart<Arc<Process>>);

impl UartWrapper{
    pub fn new(base: usize) -> Self {
        let uart = Uart::new(base);
        uart.init();
        UartWrapper(uart)
    }
}


impl CharDevice for UartWrapper {
    fn put(&self, c: u8){
        self.0.put_ch(c, |queue| {
            let task = current_process().unwrap();
            queue.push(task.clone());
            task.update_state(ProcessState::Waiting);
        }, || {
            schedule();
        })
    }
    fn get(&self) -> Option<u8> {
        self.0.get_ch(|queue| {
            let task = current_process().unwrap();
            queue.push(task.clone());
            task.update_state(ProcessState::Waiting);
        }, || {
            schedule();
        })
    }

    fn put_bytes(&self, bytes: &[u8]) {
        self.0.put_bytes(bytes, |queue| {
            let task = current_process().unwrap();
            queue.push(task.clone());
            task.update_state(ProcessState::Waiting);
        }, || {
            schedule();
        })
    }
}



impl DeviceBase for UartWrapper {
    fn hand_irq(&self) {
        self.0.hand_irq(|queue|{
            queue.into_iter().for_each(|task|{
                task.update_state(ProcessState::Ready);
                PROCESS_MANAGER.lock().push_back(task);
            })
        })
    }
}
