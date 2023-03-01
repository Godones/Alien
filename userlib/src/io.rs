use crate::syscall::sys_write;
use core::fmt::Write;

struct Stdout;
impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        sys_write(1, s.as_ptr(), s.len());
        Ok(())
    }
}

#[doc(hidden)]
pub fn __print(args: core::fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
/// print string macro
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::io::__print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
/// println string macro
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::io::__print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}
