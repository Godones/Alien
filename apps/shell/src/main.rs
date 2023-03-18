#![no_std]
#![no_main]

extern crate alloc;

mod executor;

use Mstd::{print, println};
use executor::Executor;

const BEGAN: &str = r#"
    Modular OS Shell
    Type 'help' for a list of commands
    "#;


#[no_mangle]
fn main() -> isize {
    println!("{}", BEGAN);
    loop {
        print!("> ");
        let str = Mstd::io::read_line();
        let executor = Executor::new(&str);
        executor.run();
    }
}

