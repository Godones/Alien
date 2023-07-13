use core::fmt::{Arguments, Result, Write};
use core::sync::atomic::{AtomicBool, Ordering};

use preprint::Print;
use spin::Lazy;

use kernel_sync::Mutex;

use crate::driver::uart::{CharDevice, USER_UART};
use crate::sbi::console_putchar;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        let hard_id = $crate::arch::hart_id();
        // [hart_id] xxx
        $crate::print::console::__print(format_args!("[{}] {}", hard_id, format_args!($($arg)*)))
    };
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::print!(
        concat!($fmt, "\n"), $($arg)*));
}

#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => {
        $crate::print::console::__print(format_args!("{}", format_args!($($arg)*)))
    };
}

#[macro_export]
macro_rules! eprintln {
    () => ($crate::eprint!("\n"));
    ($fmt:expr) => ($crate::eprint!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::eprint!(
        concat!($fmt, "\n"), $($arg)*));
}

#[macro_export]
macro_rules! uprint {
   ($($arg:tt)*) => {
        $crate::print::console::__uprint(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! uprintln {
     () => ($crate::uprint!("\n"));
    ($fmt:expr) => ($crate::uprint!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::uprint!(
        concat!($fmt, "\n"), $($arg)*));
}

struct Stdout;

static STDOUT: Lazy<Mutex<Stdout>> = Lazy::new(|| Mutex::new(Stdout));

pub static UART_FLAG: AtomicBool = AtomicBool::new(false);

/// 对`Stdout`实现输出的Trait
impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> Result {
        if UART_FLAG.load(Ordering::Relaxed) {
            let uart = USER_UART.get().unwrap();
            uart.put_bytes(s.as_bytes());
        } else {
            s.as_bytes().iter().for_each(|x| {
                console_putchar(*x);
            });
        }

        Ok(())
    }
}

pub fn get_char() -> Option<u8> {
    let uart = USER_UART.get().unwrap();
    uart.get()
}

pub fn check_have_char() -> bool {
    let uart = USER_UART.get().unwrap();
    uart.have_data_to_get()
}

pub fn check_have_space() -> bool {
    let uart = USER_UART.get().unwrap();
    uart.have_space_to_put()
}

/// 输出函数
/// 对参数进行输出 主要使用在输出相关的宏中 如println
pub fn __print(args: Arguments) {
    STDOUT.lock().write_fmt(args).unwrap();
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
