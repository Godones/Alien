#[macro_use]
pub mod console;
mod logging;
mod uart;

use self::uart::Ns16550a;
pub use console::PrePrint;
pub use logging::init_logger;
use spin::Mutex;

pub trait Uart {
    fn put(&mut self, c: u8);
    fn get(&mut self) -> Option<u8>;
}

pub static STDOUT: Mutex<Ns16550a> = Mutex::new(Ns16550a);
