pub struct Uart8250Raw<const W: usize> {
    base: usize,
}

impl<const W: usize> Uart8250Raw<W> {
    pub const fn new(base: usize) -> Self {
        Self { base }
    }
    pub fn init(&self) {
        // enable receive interrupts
        let base = self.base as *mut u8;
        unsafe {
            let ier = base.add(1 * W).read_volatile();
            base.add(1 * W).write_volatile(ier | 0x01);
        }
    }
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
