use interface::{Basic, Uart};
use rref::RpcResult;
use std::sync::Arc;
#[derive(Debug)]
pub struct UartDomain {}

impl UartDomain {
    pub fn new(_uart_addr: usize) -> Self {
        Self {}
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
}

pub fn main(uart_addr: usize) -> Arc<dyn Uart> {
    libsyscall::println!("uart_addr: {:#x}", uart_addr);
    Arc::new(UartDomain::new(uart_addr))
}
