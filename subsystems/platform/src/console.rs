use core::fmt::{Arguments, Result, Write};
use preprint::Print;
use ksync::Mutex;
/// 系统启动初期使用的输出函数
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        let hard_id = arch::hart_id();
        $crate::console::__print(format_args!("[{}] {}", hard_id, format_args!($($arg)*)))
    };
}

/// 系统启动初期使用的输出函数
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::print!(
        concat!($fmt, "\n"), $($arg)*));
}

pub struct Stdout;

/// 对`Stdout`实现输出的Trait
impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> Result {
        s.as_bytes().iter().for_each(|x| {
            crate::console_putchar(*x);
        });
        Ok(())
    }
}

static STDOUT: Mutex<Stdout> = Mutex::new(Stdout);

/// 输出函数
/// 对参数进行输出 主要使用在输出相关的宏中 如println
#[doc(hidden)]
pub fn __print(args: Arguments) {
    STDOUT.lock().write_fmt(args).unwrap();
}

/// 系统启动初期使用的输出函数
///
/// 在riscv平台上，由于没有实现串口驱动，所以在系统启动初期使用SBI进行输出
pub fn console_write(s: &str) {
    Stdout.write_str(s).unwrap();
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