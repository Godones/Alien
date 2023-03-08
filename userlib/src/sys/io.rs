use core::fmt::Write;
use core2::io::Read;
use crate::syscall::{sys_read, sys_write};

type Result<T> = core2::io::Result<T>;
pub type Stderr = Stdout;

#[derive(Debug)]
pub struct Stdout;
impl Stdout {
    pub fn new() -> Self {
        Stdout {}
    }
}
impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        sys_write(1, s.as_ptr(), s.len());
        Ok(())
    }
}
#[derive(Debug)]
pub struct Stdin;

impl Stdin {
    pub fn new() -> Self {
        Stdin {}
    }
}
impl Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let len = sys_read(0, buf.as_mut_ptr(), buf.len());
        Ok(len as usize)
    }
}

impl core2::io::Write for Stdout{
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let len = sys_write(1, buf.as_ptr(), buf.len());
        Ok(len as usize)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

