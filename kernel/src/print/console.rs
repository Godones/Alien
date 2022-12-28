use super::{Uart, UART};
use alloc::string::String;
use core::fmt::{Arguments, Result, Write};

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

struct Stdout;

/// 对`Stdout`实现输出的Trait
impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> Result {
        let mut buffer = [0u8; 4];
        let mut stdout = UART.lock();
        for c in s.chars() {
            for code_point in c.encode_utf8(&mut buffer).as_bytes().iter() {
                stdout.put(*code_point);
            }
        }
        Ok(())
    }
}

pub fn get_char() -> Option<u8> {
    UART.lock().get()
}

pub fn get_line() -> String {
    let mut line = String::new();
    loop {
        match get_char() {
            Some(ch) => {
                match ch {
                    //退格键或者删除键
                    8 | 0x7f => {
                        if line.len() > 0 {
                            line.pop();
                            print!("\x08 \x08");
                        }
                    }
                    //回车键
                    13 => {
                        println!(""); //换行
                        return line;
                    }
                    _ => {
                        line.push(ch as char);
                        print!("{}", ch as char);
                    }
                }
            }
            None => {}
        }
    }
}

/// 输出函数
/// 对参数进行输出 主要使用在输出相关的宏中 如println
pub fn __print(args: Arguments) {
    Stdout.write_fmt(args).unwrap();
}

use preprint::Print;

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
