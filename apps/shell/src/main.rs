#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::ToString;

use executor::Executor;
use Mstd::{print, println};

use crate::executor::CURRENT_DIR;

mod executor;

const BEGAN: &str = r#"
    Alien OS Shell
    Type 'help' for a list of commands
    "#;

#[no_mangle]
fn main() -> isize {
    println!("{}", BEGAN);
    *CURRENT_DIR.lock() = Some("/".to_string());
    loop {
        print!("{} > ", CURRENT_DIR.lock().as_ref().unwrap().as_str());
        let str = Mstd::io::read_line();
        let executor = Executor::new(&str);
        executor.run();
    }
}
