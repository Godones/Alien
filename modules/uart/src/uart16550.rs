/// The UART 16550 driver.
pub struct Uart16550Raw(usize);

impl Uart16550Raw {
    pub fn new(base: usize) -> Self {
        Uart16550Raw(base)
    }
    /// Initialize the UART.
    pub fn init(&self) {
        let ptr = self.0 as *mut u8;
        unsafe {
            //disable interrupts
            ptr.add(1).write_volatile(0);
            // // special mode to set baud rate.
            ptr.add(3).write_volatile(0x80);
            // // LSB for baud rate of 38.4K
            ptr.add(0).write_volatile(0x03);
            // // MSB for baud rate of 38.4k
            ptr.add(1).write_volatile(0x00);
            // // set the world length to 8 bits
            ptr.add(3).write_volatile(3);
            // reset and enable FIFOs.
            ptr.add(2).write_volatile(0x7);
            // enable receive interrupts and transmit interrupts
            ptr.add(1).write_volatile(0x1);
            // ptr.add(1).write_volatile(0x1);
        }
    }
    /// Write a byte to the UART.
    pub fn put(&self, c: u8) {
        let ptr = self.0 as *mut u8;
        unsafe {
            // wait for transmitter to be ready
            while ptr.add(5).read_volatile() & 0x20 == 0 {}
            // write
            ptr.add(0).write_volatile(c);
        }
    }
    /// Read a byte from the UART. If there is no data, return `None`.
    pub fn read(&self) -> Option<u8> {
        let ptr = self.0 as *mut u8;
        unsafe {
            // check if there is data
            if ptr.add(5).read_volatile() & 1 == 0 {
                None
            } else {
                Some(ptr.add(0).read_volatile())
            }
        }
    }
}
