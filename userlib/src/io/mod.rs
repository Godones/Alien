mod stdio;
use alloc::string::String;
use core::fmt;
use core2::io::{BufRead, Read, Write};
use stdio::*;

pub use stdio::{stdin, stdout, StdinLock, StdoutLock};

type Result<T> = core2::io::Result<T>;

pub trait BufferReadExt {
    fn read_line(&mut self, buf: &mut String) -> Result<usize>;
    fn lines(self) -> Lines<Self>
    where
        Self: Sized,
    {
        Lines { buf: self }
    }
}

impl<T: BufRead> BufferReadExt for T {
    fn read_line(&mut self, buf: &mut String) -> Result<usize> {
        unsafe { self.read_to_end(buf.as_mut_vec()) }
    }
}

pub struct Lines<B> {
    buf: B,
}

impl<B: BufRead> Iterator for Lines<B> {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Result<String>> {
        let mut buf = String::new();
        match self.buf.read_line(&mut buf) {
            Ok(0) => None,
            Ok(_n) => {
                if buf.ends_with('\n') {
                    buf.pop();
                    if buf.ends_with('\r') {
                        buf.pop();
                    }
                }
                Some(Ok(buf))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::io::_print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::io::_print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

fn print_to<T>(args: fmt::Arguments<'_>, global_s: fn() -> T, label: &str)
where
    T: Write,
{
    if let Err(e) = global_s().write_fmt(args) {
        panic!("failed printing to {label}: {e}");
    }
}

fn use_raw_stdout(args: fmt::Arguments<'_>) {
    use super::sys::io::Stdout;
    Stdout.write_fmt(args).unwrap();
}

#[doc(hidden)]
#[cfg(not(test))]
pub fn _print(args: fmt::Arguments<'_>) {
    // print_to(args, stdout, "stdout");
    use_raw_stdout(args);
}

#[doc(hidden)]
#[cfg(not(test))]
pub fn _eprint(args: fmt::Arguments<'_>) {
    print_to(args, stdout, "stderr");
}

pub fn read_line() -> String {
    use super::sys::io::Stdin;
    let mut buf = [0u8; 1];
    let mut res = String::new();
    loop {
        Stdin.read(&mut buf).unwrap();
        if buf[0] == b'\n' || buf[0] == b'\r' {
            break;
        }
        if buf[0] == 127 {
            if res.len() > 0 {
                print!("\x08 \x08");
                res.pop();
            }
            continue;
        }
        if buf[0] < 32 {
            continue;
        }
        print!("{}", buf[0] as char);
        res.push(buf[0] as char);
    }
    print!("\n");
    res
}
