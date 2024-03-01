use interface::{Basic, Uart};
use region::SafeRegion;
use rref::RpcResult;
use std::sync::Arc;

#[derive(Debug)]
pub struct UartDomain {
    region: SafeRegion,
}

impl UartDomain {
    pub fn new(uart_addr: usize, size: usize) -> Self {
        Self {
            region: SafeRegion::new(uart_addr, size).unwrap(),
        }
    }
}

impl Basic for UartDomain {}

impl Uart for UartDomain {
    fn putc(&self, _ch: u8) -> RpcResult<()> {
        todo!()
    }

    fn getc(&self) -> RpcResult<Option<u8>> {
        todo!()
    }

    fn handle_irq(&self) -> RpcResult<()> {
        todo!()
    }
}

pub fn main(uart_addr: usize, size: usize) -> Arc<dyn Uart> {
    libsyscall::println!("uart_addr: {:#x}-{:#x}", uart_addr, uart_addr + size);
    Arc::new(UartDomain::new(uart_addr, size))
}
