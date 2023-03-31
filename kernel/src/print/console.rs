use crate::driver::uart::{CharDevice, UART, USER_UART};
use core::fmt::{Arguments, Result, Write};
use preprint::Print;

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::print::console::__print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::print::console::__print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! uprint {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::print::console::__uprint(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! uprintln {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::print::console::__uprint(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

struct Stdout;

/// 对`Stdout`实现输出的Trait
impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> Result {
        UART.lock().write_str(s)
    }
}

pub fn get_char() -> Option<u8> {
    let uart = USER_UART.get().unwrap();
    uart.get()
}

/// 输出函数
/// 对参数进行输出 主要使用在输出相关的宏中 如println
pub fn __print(args: Arguments) {
    Stdout.write_fmt(args).unwrap();
}

struct UStdout;

impl Write for UStdout {
    fn write_str(&mut self, out: &str) -> Result {
        let uart = USER_UART.get().unwrap();
        uart.put_bytes(out.as_bytes());
        Ok(())
    }
}

pub fn __uprint(args: Arguments) {
    UStdout.write_fmt(args).unwrap();
}

pub struct PrePrint;

impl Print for PrePrint {
    fn print(&self, args: Arguments) {
        print!("{}", args);
    }
}
impl Write for PrePrint {
    fn write_str(&mut self, s: &str) -> Result {
        print!("{}", s);
        Ok(())
    }
}
