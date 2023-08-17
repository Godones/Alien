/// The UART 8250 driver.
pub struct Uart8250Raw<const W: usize> {
    base: usize,
}

impl<const W: usize> Uart8250Raw<W> {
    pub const fn new(base: usize) -> Self {
        Self { base }
    }
    /// Initialize the UART.
    ///
    /// It will enable receive interrupts.
    pub fn init(&self) {
        // enable receive interrupts
        let base = self.base as *mut u8;
        unsafe {
            let ier = base.add(1 * W).read_volatile();
            base.add(1 * W).write_volatile(ier | 0x01);
        }
    }
    /// Write a byte to the UART.
    pub fn put(&mut self, c: u8) {
        let base = self.base as *mut u8;
        loop {
            let lsr = unsafe { base.add(5 * W).read_volatile() };
            if lsr & 0x20 != 0 {
                break;
            }
        }
        unsafe {
            base.add(W * 0).write_volatile(c);
        }
    }
    /// Read a byte from the UART. If there is no data, return `None`.
    pub fn read(&mut self) -> Option<u8> {
        let base = self.base as *mut u8;
        let lsr = unsafe { base.add(5 * W).read_volatile() };
        if lsr & 0x01 != 0 {
            Some(unsafe { base.add(0 * W).read_volatile() })
        } else {
            None
        }
    }
}
