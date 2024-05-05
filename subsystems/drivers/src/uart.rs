use alloc::{boxed::Box, collections::VecDeque, sync::Arc};

use device_interface::{DeviceBase, UartDevice};
use ksync::Mutex;
use shim::KTask;

pub use self::{uart16550::Uart16550, uart8250::Uart8250};

pub trait LowUartDriver: Send + Sync {
    fn _init(&mut self);
    fn _put(&mut self, c: u8);
    fn _read(&mut self) -> Option<u8>;
}

mod uart8250 {
    use crate::uart::LowUartDriver;

    pub struct Uart8250 {
        uart_raw: uart8250::MmioUart8250<'static, u32>,
    }

    unsafe impl Send for Uart8250 {}

    unsafe impl Sync for Uart8250 {}

    impl Uart8250 {
        pub fn new(base_addr: usize) -> Self {
            let uart_raw = unsafe { uart8250::MmioUart8250::<u32>::new(base_addr) };
            Uart8250 { uart_raw }
        }
    }

    impl LowUartDriver for Uart8250 {
        fn _init(&mut self) {
            self.uart_raw.enable_received_data_available_interrupt();
        }

        fn _put(&mut self, c: u8) {
            loop {
                if self.uart_raw.write_byte(c).is_ok() {
                    break;
                }
            }
        }

        fn _read(&mut self) -> Option<u8> {
            self.uart_raw.read_byte()
        }
    }
}

mod uart16550 {
    use crate::uart::LowUartDriver;

    pub struct Uart16550 {
        uart_raw: &'static mut uart16550::Uart16550<u8>,
    }

    unsafe impl Send for Uart16550 {}

    unsafe impl Sync for Uart16550 {}

    impl Uart16550 {
        pub fn new(base_addr: usize) -> Self {
            let uart_raw = unsafe { &mut *(base_addr as *mut uart16550::Uart16550<u8>) };
            Uart16550 { uart_raw }
        }
    }

    impl LowUartDriver for Uart16550 {
        fn _init(&mut self) {
            use uart16550::InterruptTypes;
            let ier = self.uart_raw.ier();
            let inter = InterruptTypes::ZERO;
            ier.write(inter.enable_rda());
        }

        fn _put(&mut self, c: u8) {
            self.uart_raw.write(&[c]);
        }

        fn _read(&mut self) -> Option<u8> {
            let mut buf = [0];
            let r = self.uart_raw.read(&mut buf);
            if r == 0 {
                None
            } else {
                Some(buf[0])
            }
        }
    }
}

pub struct Uart {
    inner: Mutex<(Box<dyn LowUartDriver>, UartInner)>,
}

struct UartInner {
    rx_buf: VecDeque<u8>,
    wait_queue: VecDeque<Arc<dyn KTask>>,
}

impl Uart {
    pub fn new(uart_raw: Box<dyn LowUartDriver>) -> Self {
        let mut uart_raw = uart_raw;
        uart_raw._init();
        let inner = UartInner {
            rx_buf: VecDeque::new(),
            wait_queue: VecDeque::new(),
        };
        Uart {
            inner: Mutex::new((uart_raw, inner)),
        }
    }
}

impl UartDevice for Uart {
    fn put(&self, c: u8) {
        let mut inner = self.inner.lock();
        inner.0._put(c);
    }
    fn get(&self) -> Option<u8> {
        loop {
            let mut inner = self.inner.lock();
            if inner.1.rx_buf.is_empty() {
                let task = shim::take_current_task().unwrap();
                task.to_wait();
                inner.1.wait_queue.push_back(task.clone());
                drop(inner);
                shim::schedule_now(task);
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

impl DeviceBase for Uart {
    fn handle_irq(&self) {
        loop {
            let mut inner = self.inner.lock();
            if let Some(c) = inner.0._read() {
                inner.1.rx_buf.push_back(c);
                if !inner.1.wait_queue.is_empty() {
                    let task = inner.1.wait_queue.pop_front().unwrap();
                    task.to_wakeup();
                    shim::put_task(task);
                }
            } else {
                break;
            }
        }
    }
}
