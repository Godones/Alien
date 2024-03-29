use alloc::boxed::Box;

use constants::{AlienError, AlienResult};
use interface::{Basic, BufUartDomain, DeviceBase};

#[derive(Debug)]
pub struct BufUartDomainProxy {
    id: u64,
    domain: Box<dyn BufUartDomain>,
}

impl BufUartDomainProxy {
    pub fn new(id: u64, domain: Box<dyn BufUartDomain>) -> Self {
        Self { id, domain }
    }
}

impl DeviceBase for BufUartDomainProxy {
    fn handle_irq(&self) -> AlienResult<()> {
        if !self.domain.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.handle_irq()
    }
}

impl Basic for BufUartDomainProxy {
    fn is_active(&self) -> bool {
        self.domain.is_active()
    }
}

impl BufUartDomain for BufUartDomainProxy {
    fn init(&self, uart_domain_name: &str) -> AlienResult<()> {
        self.domain.init(uart_domain_name)
    }

    fn putc(&self, ch: u8) -> AlienResult<()> {
        if !self.domain.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.putc(ch)
    }

    fn getc(&self) -> AlienResult<Option<u8>> {
        if !self.domain.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.getc()
    }

    fn have_data_to_get(&self) -> AlienResult<bool> {
        if !self.domain.is_active() {
            return Err(AlienError::DOMAINCRASH);
        }
        self.domain.have_data_to_get()
    }
}
