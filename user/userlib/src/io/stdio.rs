use super::*;
use crate::sys::DEFAULT_BUFFER_SIZE;
use alloc::string::String;
use core::fmt;
use core2::io::{BufReader, BufWriter, LineWriter, Read, Write};
use spin::{Mutex, MutexGuard, Once};

type RawStdin = crate::sys::io::Stdin;
type RawStdout = crate::sys::io::Stdout;

#[derive(Debug)]
pub struct Stdin {
    inner: &'static Mutex<BufReader<RawStdin, DEFAULT_BUFFER_SIZE>>,
}

pub fn stdin() -> Stdin {
    static INSTANCE: Once<Mutex<BufReader<RawStdin, DEFAULT_BUFFER_SIZE>>> = Once::new();
    Stdin {
        inner: INSTANCE.call_once(|| Mutex::new(BufReader::new(RawStdin::new()))),
    }
}

impl Stdin {
    pub fn lock(&self) -> StdinLock<'static> {
        StdinLock {
            inner: self.inner.lock(),
        }
    }

    pub fn read_line(&self, buf: &mut String) -> Result<usize> {
        self.lock().read_line(buf)
    }

    pub fn lines(self) -> Lines<StdinLock<'static>> {
        self.lock().lines()
    }
}

impl Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.lock().read(buf)
    }
}

pub struct StdinLock<'a> {
    inner: MutexGuard<'a, BufReader<RawStdin, DEFAULT_BUFFER_SIZE>>,
}

impl Read for StdinLock<'_> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.inner.read(buf)
    }
}

impl BufRead for StdinLock<'_> {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        self.inner.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.inner.consume(amt)
    }
}

static STDOUT_INSTANCE: Once<Mutex<LineWriter<RawStdout, 1024>>> = Once::new();

#[derive(Debug)]
pub struct Stdout {
    inner: &'static Mutex<LineWriter<RawStdout, 1024>>,
}
pub fn stdout() -> Stdout {
    Stdout {
        inner: STDOUT_INSTANCE.call_once(|| Mutex::new(LineWriter::new(RawStdout::new()))),
    }
}
pub struct StdoutLock<'a> {
    inner: MutexGuard<'a, LineWriter<RawStdout, 1024>>,
}
impl Stdout {
    pub fn lock(&self) -> StdoutLock<'static> {
        StdoutLock {
            inner: self.inner.lock(),
        }
    }
    pub fn write(&self, buf: &[u8]) -> Result<usize> {
        self.lock().write(buf)
    }
    pub fn flush(&self) -> Result<()> {
        self.lock().flush()
    }
}

impl Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.lock().write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.lock().flush()
    }
}

impl core2::io::Write for StdoutLock<'_> {
    fn write(&mut self, buf: &[u8]) -> core2::io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> core2::io::Result<()> {
        self.inner.flush()
    }
}
