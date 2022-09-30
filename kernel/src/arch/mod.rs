use spin::Mutex;
mod riscv;
use riscv::Ns16550a;

pub trait Uart {
    fn put(&mut self, c: u8);
    fn get(&mut self) -> Option<u8>;
}

pub static STDOUT: Mutex<Ns16550a> = Mutex::new(Ns16550a);
