#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![allow(unused)]
extern crate alloc;

use alloc::boxed::Box;
use core::fmt::Debug;

pub trait Uart16550IO: Debug + Send + Sync {
    fn read_at(&self, offset: usize) -> Result<u8, ()>;
    fn write_at(&self, offset: usize, value: u8) -> Result<(), ()>;
}

const RHR: usize = 0x00;
const THR: usize = 0x00;
const IER: usize = 0x01;
const IIR: usize = 0x02;
const FCR: usize = 0x02;
const LCR: usize = 0x03;
const LSR: usize = 0x05;

#[derive(Debug)]
pub struct Uart16550 {
    region: Box<dyn Uart16550IO>,
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

    pub fn new(io: Box<dyn Uart16550IO>) -> Self {
        let res = Self { region: io };
        res
    }
}

impl Uart16550 {
    pub fn enable_receive_interrupt(&self) {
        // set IER to 1
        self.region.write_at(IER, 1).unwrap();
    }

    pub fn disable_receive_interrupt(&self) {
        // set IER to 0
        self.region.write_at(IER, 0).unwrap();
    }
    pub fn putc(&self, ch: u8) {
        // check LCR DLAB = 0
        // check LSR empty
        loop {
            let lsr = self.region.read_at(LSR).unwrap();
            if (lsr & 0b10_0000) == 0b10_0000 {
                // send
                self.region.write_at(THR, ch).unwrap();
                break;
            }
        }
    }

    pub fn getc(&self) -> Option<u8> {
        // check LCR DLAB = 0
        // check LSR
        let lsr = self.region.read_at(LSR).unwrap();
        if (lsr & 1) == 1 {
            // read
            Some(self.region.read_at(THR).unwrap())
        } else {
            None
        }
        // read from RHR
    }

    pub fn have_data_to_get(&self) -> bool {
        let lsr = self.region.read_at(LSR).unwrap();
        (lsr & 1) == 1
    }
}
