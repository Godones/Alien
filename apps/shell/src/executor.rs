use alloc::string::{String, ToString};
use alloc::vec::Vec;

use spin::Mutex;

use Mstd::fs::{chdir, get_cwd};
use Mstd::process::{exec, exit, fork, waitpid};
use Mstd::thread::m_yield;
use Mstd::{println, system_shutdown};

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
        let args = parse_input(s);
        let args = args
            .args
            .into_iter()
            .map(|mut arg| {
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
struct ParsedArgs {
    args: Vec<String>,
}

impl ParsedArgs {
    fn new() -> Self {
        ParsedArgs { args: Vec::new() }
    }

    fn add_arg(&mut self, arg: &str) {
        self.args.push(arg.to_string());
    }
}

fn parse_input(input_string: &str) -> ParsedArgs {
    let mut parsed_args = ParsedArgs::new();
    let mut current_arg = "".to_string();
    let mut in_quote = false;

    for c in input_string.chars() {
        match c {
            ' ' if !in_quote => {
                if !current_arg.is_empty() {
                    parsed_args.add_arg(&current_arg);
                    current_arg = "".to_string();
                }
            }
            '=' if in_quote => current_arg.push(c),
            '"' => in_quote = !in_quote,
            _ => current_arg.push(c),
        }
    }
    if !current_arg.is_empty() {
        parsed_args.add_arg(&current_arg);
    }
    parsed_args
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

    pub fn run(mut self) {
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
            "exit" => system_shutdown(),
            _ => {
                let mut cmd = self.cmd;
                cmd.push('\0');
                if !cmd.starts_with("/") && !cmd.starts_with("./") {
                    cmd.insert_str(0, "/bin/");
                }
                let env = if cmd.ends_with("bash\0") {
                    BASH_ENV
                } else {
                    &[0 as *const u8]
                };
                let pid = fork();
                if pid == 0 {
                    self.parameter.args.insert(0, cmd.clone());
                    exec(cmd.as_str(), self.parameter.get_args_raw().as_slice(), env);
                    exit(0);
                } else {
                    m_yield();
                    let mut exit_code: i32 = 0;
                    let x = waitpid(pid as usize, &mut exit_code);
                    println!("waittid: {}, exit_code: {}", x, exit_code);
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

const BASH_ENV: &[*const u8] = &[
    "SHELL=/bash\0".as_ptr(),
    "PWD=/\0".as_ptr(),
    "LOGNAME=root\0".as_ptr(),
    "MOTD_SHOWN=pam\0".as_ptr(),
    "HOME=/root\0".as_ptr(),
    "LANG=C.UTF-8\0".as_ptr(),
    "TERM=vt220\0".as_ptr(),
    "USER=root\0".as_ptr(),
    "SHLVL=0\0".as_ptr(),
    "OLDPWD=/root\0".as_ptr(),
    "PS1=\x1b[1m\x1b[32mAlien\x1b[0m:\x1b[1m\x1b[34m\\w\x1b[0m\\$ \0".as_ptr(),
    "_=/bin/bash\0".as_ptr(),
    "PATH=/:/bin\0".as_ptr(),
    "LD_LIBRARY_PATH=/\0".as_ptr(),
    core::ptr::null(),
];
