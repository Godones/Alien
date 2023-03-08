#![no_std]
#![no_main]

mod parameter;

extern crate alloc;

use Mstd::{print, println};
use Mstd::process::{exec, fork, waitpid};
use Mstd::thread::m_yield;
use Mstd::shutdown;
#[no_mangle]
fn main() ->isize {
    println!("{}", BEGAN);
    loop {
        print!("> ");
        let str = Mstd::io::read_line();
        let (cmd, args) = str.split_once(' ').unwrap_or((&str, ""));
        execute(cmd, args);
    }
}

pub fn execute(cmd: &str, args: &str) {
    match cmd {
        "echo" => println!("{}", args),
        "help" => println!("{}", HELP),
        "exit" => shutdown(),
        _ => {
            let pid = fork();
            if pid == 0 {
                exec(cmd);
            } else {
                m_yield();
                let mut exit_code: i32 = 0;
                let _ = waitpid(pid as usize, &mut exit_code);

            }
        }
    }
}

const BEGAN: &str = r#"
    Modular OS Shell
    Type 'help' for a list of commands
    "#;
const HELP: &str = r#"
    help - Display this message
    echo - Echo the arguments
    exit - Exit the shell
    "#;