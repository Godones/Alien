use interface::{Basic, UartDomain};
use region::SafeIORegion;
use rref::RpcResult;
use std::sync::Arc;

#[derive(Debug)]
pub struct UartDomainImpl {
    region: SafeIORegion,
}

impl UartDomainImpl {
    pub fn new(uart_addr: usize, size: usize) -> Self {
        Self {
            region: SafeIORegion::new(uart_addr, size).unwrap(),
        }
    }
}

impl Basic for UartDomainImpl {}

impl UartDomain for UartDomainImpl {
    fn putc(&self, _ch: u8) -> RpcResult<()> {
        todo!()
    }

    fn getc(&self) -> RpcResult<Option<u8>> {
        todo!()
    }

    fn have_data_to_get(&self) -> bool {
        todo!()
    }

    fn handle_irq(&self) -> RpcResult<()> {
        todo!()
    }
}

pub fn main(uart_addr: usize, size: usize) -> Arc<dyn UartDomain> {
    libsyscall::println!("uart_addr: {:#x}-{:#x}", uart_addr, uart_addr + size);
    Arc::new(UartDomain::new(uart_addr, size))
}
