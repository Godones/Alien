use alloc::borrow::ToOwned;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use spin::Mutex;

use Mstd::fs::{chdir, get_cwd};
use Mstd::process::{exec, exit, fork, waitpid};
use Mstd::thread::m_yield;
use Mstd::{println, shutdown};

#[derive(Debug)]
pub struct Parameter {
    args: Vec<String>,
}

impl Parameter {
    #[allow(unused)]
    fn new() -> Self {
        Self { args: Vec::new() }
    }
    fn from_str(s: &str) -> Self {
        let args = s
            .split_whitespace()
            .map(|arg| {
                let mut arg = arg.to_owned();
                arg.push('\0');
                arg
            })
            .collect::<Vec<String>>();
        Self { args }
    }
    fn get_args_raw(&self) -> Vec<*const u8> {
        let mut raw_points = self
            .args
            .iter()
            .map(|arg| arg.as_ptr())
            .collect::<Vec<*const u8>>();
        raw_points.push(0 as *const u8);
        raw_points
    }
}

#[derive(Debug)]
pub struct Executor {
    parameter: Parameter,
    cmd: String,
}

impl Executor {
    pub fn new(input: &str) -> Self {
        let (cmd, args) = input.split_once(' ').unwrap_or((input, ""));
        let cmd = cmd.to_string();
        Self {
            parameter: Parameter::from_str(args),
            cmd,
        }
    }

    pub fn run(self) {
        if self.cmd.is_empty() {
            return;
        }
        match self.cmd.as_str() {
            "echo" => println!("{}", self.parameter.args.join(" ")),
            "help" => println!("{}", HELP),
            "cd" => {
                println!("cd to {}", self.parameter.args[0]);
                let res = chdir(&self.parameter.args[0]);
                if res == -1 {
                    println!("chdir failed");
                } else {
                    let mut buf = [0u8; 50];
                    let cwd = get_cwd(&mut buf).unwrap();
                    println!("chdir success, now cwd: {}", cwd);
                    *CURRENT_DIR.lock() = Some(cwd.to_string());
                }
            }
            "exit" => shutdown(),
            _ => {
                let mut cmd = self.cmd;
                cmd.push('\0');
                let pid = fork();
                if pid == 0 {
                    // self.parameter.args.insert(0, cmd.clone());
                    exec(
                        cmd.as_str(),
                        self.parameter.get_args_raw().as_slice(),
                        &[0 as *const u8],
                    );
                    exit(0);
                } else {
                    m_yield();
                    let mut exit_code: i32 = 0;
                    let x = waitpid(pid as usize, &mut exit_code);
                    println!("waitpid: {}, exit_code: {}", x, exit_code);
                }
            }
        }
    }
}

pub static CURRENT_DIR: Mutex<Option<String>> = Mutex::new(None);

const HELP: &str = r#"
    help - Display this message
    echo - Echo the arguments
    exit - Exit the shell
    "#;
