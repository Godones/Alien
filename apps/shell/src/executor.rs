use alloc::borrow::ToOwned;
use Mstd::process::{exec, fork, waitpid};
use Mstd::thread::m_yield;
use alloc::string::{String};
use alloc::vec::Vec;
use Mstd::{println, shutdown};

#[derive(Debug)]
pub struct Parameter{
    args:Vec<String>,
}


impl Parameter {
    #[allow(unused)]
    fn new() -> Self {
        Self {
            args: Vec::new(),
        }
    }
    fn from_str(s:&str) -> Self {
        let args = s
            .split_whitespace()
            .map(|arg|{
                let mut arg = arg.to_owned();
                arg.push('\0');
                arg
            }).collect::<Vec<String>>();
        Self {
            args,
        }
    }
    fn get_args_raw(&self) -> Vec<*const u8> {
        let mut raw_points = self.args.iter()
            .map(|arg| arg.as_ptr())
            .collect::<Vec<*const u8>>();
        raw_points.push(0 as *const u8);
        raw_points
    }
}
#[derive(Debug)]
pub struct Executor{
    parameter:Parameter,
    cmd:String,
}

impl Executor{
    pub fn new(input:&str)->Self{
        let (cmd, args) = input.split_once(' ').unwrap_or((input, ""));
        let mut cmd = cmd.to_owned();
        cmd.push('\0');
        Self{
            parameter:Parameter::from_str(args),
            cmd,
        }
    }

    pub fn run(self){
        if self.cmd.is_empty(){
            return;
        }
        match self.cmd.as_str() {
            "echo" => println!("{}", self.parameter.args.join(" ")),
            "help" => println!("{}", HELP),
            "exit" => shutdown(),
            _ => {
                let pid = fork();
                if pid == 0 {
                    exec(self.cmd.as_str(),self.parameter.get_args_raw().as_slice());
                } else {
                    m_yield();
                    let mut exit_code: i32 = 0;
                    let _ = waitpid(pid as usize, &mut exit_code);
                }
            }
        }
    }
}


const HELP: &str = r#"
    help - Display this message
    echo - Echo the arguments
    exit - Exit the shell
    "#;