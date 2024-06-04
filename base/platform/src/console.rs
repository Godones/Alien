use core::fmt::{Arguments, Result, Write};

use spin::Mutex;

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

#[macro_export]
macro_rules! iprint {
    ($($arg:tt)*) => {
        $crate::console::__print(format_args!("{}", format_args!($($arg)*)))
    };
}

pub struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> Result {
        s.as_bytes().iter().for_each(|x| {
            crate::console_putchar(*x);
        });
        Ok(())
    }
}

static K_STDOUT: Mutex<Stdout> = Mutex::new(Stdout);

pub fn __print(args: Arguments) {
    K_STDOUT.lock().write_fmt(args).unwrap();
}
