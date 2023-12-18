use core::fmt::{Arguments, Result, Write};
use core::sync::atomic::AtomicBool;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        let hard_id = arch::hart_id();
        $crate::console::__print(format_args!("[{}] {}", hard_id, format_args!($($arg)*)))
    };
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::print!(
        concat!($fmt, "\n"), $($arg)*));
}

pub struct Stdout;

pub static UART_FLAG: AtomicBool = AtomicBool::new(false);

/// 对`Stdout`实现输出的Trait
impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> Result {
        // if UART_FLAG.load(Ordering::Relaxed) {
        //     let uart = UART_DEVICE.get().unwrap();
        //     uart.put_bytes(s.as_bytes());
        // } else {
        //
        // }
        s.as_bytes().iter().for_each(|x| {
            crate::console_putchar(*x);
        });
        Ok(())
    }
}

/// 输出函数
/// 对参数进行输出 主要使用在输出相关的宏中 如println
pub fn __print(args: Arguments) {
    Stdout.write_fmt(args).unwrap();
}
