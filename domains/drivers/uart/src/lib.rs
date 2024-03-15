#![no_std]
extern crate alloc;
extern crate malloc;

use alloc::sync::Arc;
use interface::{Basic, DeviceBase, DeviceInfo, UartDomain};
use region::SafeIORegion;
use rref::{RRef, RRefVec, RpcResult};

#[derive(Debug)]
pub struct Uart16550 {
    region: SafeIORegion,
}

impl Uart16550 {
    
    /*
    https://caro.su/msx/ocm_de1/16550.pdf
    Address	Register	Access Type	Reset Value Description
    0x00	    RHR	        Read only	0x00        Receiver Buffer Register
    0x00	    THR     	Write only	0x00	    Transmitter Holding Register
    0x01	    IER	        Read/Write	0x00	    Enable(1)/Disable(0) interrupts. See this for more details on each interrupt.
    0x02	    IIR	        Read only	0x01	    Information which interrupt occurred
    0x02	    FCR	        Write only	0x00	    Control behavior of the internal FIFOs. Currently writing to this Register has no effect.
    0x03	    LCR	        Read/Write	0x00	    The only bit in this register that has any meaning is LCR7 aka the DLAB, all other bits hold their written value but have no meaning.
    0x05	    LSR	        Read only	0x60	    Information about state of the UART. After the UART is reset, 0x60 indicates when it is ready to transmit data.
        0 =1 RHR ready to receive
        5 =1 THR empty to transmit
    */    

    pub fn new(uart_addr: usize, size: usize) -> Self {
        let res = Self {
            region: SafeIORegion::new(uart_addr, size).unwrap(),
        };
        // init: enable data ready interrupt by setting IER to 1
        res.region.write_at::<u8>(1, 1).unwrap();
        res
    }
}

impl Basic for Uart16550 {}

impl DeviceBase for Uart16550 {
    fn handle_irq(&self) -> RpcResult<()> {
        todo!()
    }
}

impl UartDomain for Uart16550 {
    fn putc(&self, ch: u8) -> RpcResult<()> {
        // check LCR DLAB = 0
        // check LSR empty
        let lsr = self.region.read_at::<u8>(5).unwrap();
        if (lsr & 0b10_0000) == 0b10_0000 {
            // send
            self.region.write_at::<u8>(0, ch).unwrap();
        }
        Ok(())
    }

    fn getc(&self) -> RpcResult<Option<u8>> {
        // check LCR DLAB = 0
        // check LSR
        let lsr = self.region.read_at::<u8>(5).unwrap();
        if (lsr & 1) == 1 {
            // read
            Ok(Some(self.region.read_at::<u8>(0).unwrap()))
        }
        else {
            Ok(None)
        }
        // read from RHR
    }

    fn have_data_to_get(&self) -> RpcResult<bool> {
        let lsr = self.region.read_at::<u8>(5).unwrap();
        Ok((lsr & 1) == 1)
    }
}

pub fn main() -> Arc<dyn UartDomain> {
    let devices_domain = libsyscall::get_devices_domain().unwrap();
    let name = RRefVec::from_slice("uart".as_bytes());

    let info = RRef::new(DeviceInfo {
        address_range: Default::default(),
        irq: RRef::new(0),
        compatible: RRefVec::new(0, 64),
    });

    let info = devices_domain.get_device(name, info).unwrap();
    let region = &info.address_range;
    libsyscall::println!("uart_addr: {:#x}-{:#x}", region.start, region.end);
    Arc::new(Uart16550::new(region.start, region.end - region.start))
}
