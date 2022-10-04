use core::fmt::{Write, Arguments, Result};
use crate::arch::{STDOUT, Uart};

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

struct Stdout;

/// 对`Stdout`实现输出的Trait
impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> Result {
        let mut buffer = [0u8; 4];
        for c in s.chars() {
            for code_point in c.encode_utf8(&mut buffer).as_bytes().iter() {
                STDOUT.lock().put(*code_point);
            }
        }
        Ok(())
    }
}



/// 输出函数
/// 
/// 对参数进行输出 主要使用在输出相关的宏中 如println
pub fn print(args: Arguments) {
    Stdout.write_fmt(args).unwrap();
}