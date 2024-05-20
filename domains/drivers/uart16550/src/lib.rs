#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::boxed::Box;
use core::{fmt::Debug, ops::Range};

use basic::{io::SafeIORegion, println, AlienResult};
use interface::{Basic, DeviceBase, UartDomain};
use raw_uart16550::{InterruptTypes, Uart16550, Uart16550IO};
use spin::Once;

static UART: Once<Uart16550<u8>> = Once::new();

#[derive(Debug)]
pub struct SafeIORegionWrapper(SafeIORegion);

impl Uart16550IO<u8> for SafeIORegionWrapper {
    fn read_at(&self, offset: usize) -> u8 {
        self.0.read_at(offset).unwrap()
    }

    fn write_at(&self, offset: usize, value: u8) {
        self.0.write_at(offset, value).unwrap()
    }
}

#[derive(Debug)]
struct UartDomainImpl;

impl DeviceBase for UartDomainImpl {
    fn handle_irq(&self) -> AlienResult<()> {
        todo!()
    }
}

impl Basic for UartDomainImpl {}

impl UartDomain for UartDomainImpl {
    fn init(&self, address_range: Range<usize>) -> AlienResult<()> {
        let region = &address_range;
        println!("uart_addr: {:#x}-{:#x}", region.start, region.end);
        let io_region = SafeIORegion::from(region.clone());
        let uart = Uart16550::new(Box::new(SafeIORegionWrapper(io_region)));
        UART.call_once(|| uart);
        self.enable_receive_interrupt()?;
        Ok(())
    }

    fn putc(&self, ch: u8) -> AlienResult<()> {
        UART.get().unwrap().write(&[ch]);
        Ok(())
    }

    fn getc(&self) -> AlienResult<Option<u8>> {
        let mut buf = [0];
        let c = UART.get().unwrap().read(&mut buf);
        assert!(c <= 1);
        if c == 0 {
            Ok(None)
        } else {
            Ok(Some(buf[0]))
        }
    }

    fn have_data_to_get(&self) -> AlienResult<bool> {
        let uart = UART.get().unwrap();
        let lsr = uart.lsr();
        let region = uart.io_region();
        let status = lsr.read(region);
        Ok(status.is_data_ready())
    }

    fn enable_receive_interrupt(&self) -> AlienResult<()> {
        let uart = UART.get().unwrap();
        let ier = uart.ier();
        let region = uart.io_region();
        let inter = InterruptTypes::ZERO;
        ier.write(region, inter.enable_rda());
        Ok(())
    }

    fn disable_receive_interrupt(&self) -> AlienResult<()> {
        let uart = UART.get().unwrap();
        let ier = uart.ier();
        let region = uart.io_region();
        let inter = InterruptTypes::ZERO;
        ier.write(region, inter.disable_rda());
        Ok(())
    }
}

pub fn main() -> Box<dyn UartDomain> {
    Box::new(UartDomainImpl)
}
