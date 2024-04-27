#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::{String, ToString};

use Mstd::{
    domain::{register_domain, update_domain, DomainTypeRaw},
    fs::{open, OpenFlags},
    println,
};

pub enum Command {
    Register(RegisterArgs),
    Update(UpdateArgs),
    Help,
}

pub struct RegisterArgs {
    pub name: String,
    pub ty: DomainTypeRaw,
}

impl RegisterArgs {
    pub fn new(name: String, ty: DomainTypeRaw) -> Self {
        Self { name, ty }
    }
}

pub struct UpdateArgs {
    pub old_name: String,
    pub name: String,
    pub ty: DomainTypeRaw,
}

impl UpdateArgs {
    pub fn new(old_name: String, name: String, ty: DomainTypeRaw) -> Self {
        Self { old_name, name, ty }
    }
}

pub trait Run {
    fn run(&self) -> isize;
}

impl Run for RegisterArgs {
    fn run(&self) -> isize {
        let sshadow_blk_fd = open("/tests/gsshadow_blk\0", OpenFlags::O_RDWR);
        if sshadow_blk_fd == -1 {
            println!("Failed to open /tests/gsshadow_blk");
        } else {
            println!("Opened /tests/gsshadow_blk, fd: {}", sshadow_blk_fd);
        }
        let res = register_domain(
            sshadow_blk_fd as _,
            DomainTypeRaw::ShadowBlockDomain,
            "sshadow_blk",
        );
        println!("load_domain res: {}", res);
        let res = update_domain("shadow_blk-1", "sshadow_blk");
        println!("replace_domain res: {}", res);
        0
    }
}

impl Run for UpdateArgs {
    fn run(&self) -> isize {
        let res = update_domain("shadow_blk-1", "sshadow_blk");
        println!("replace_domain res: {}", res);
        0
    }
}

impl Run for Command {
    fn run(&self) -> isize {
        match self {
            Command::Register(args) => args.run(),
            Command::Update(args) => args.run(),
            Command::Help => {
                println!("Usage: domain [register|update] [args]");
                println!("register: register a domain");
                println!("register args: [name:str] [type:str]");
                println!("update: update a domain");
                println!("update args: [old_name:str] [name:str]");
                -1
            }
        }
    }
}

#[no_mangle]
fn main() -> isize {
    let input = Mstd::io::read_line();
    let (cmd, args) = input.split_once(' ').unwrap_or((&input, ""));
    let cmd = match cmd {
        "register" => {
            let (name, _ty) = args.split_once(' ').unwrap();
            Command::Register(RegisterArgs::new(
                name.to_string(),
                DomainTypeRaw::ShadowBlockDomain,
            ))
        }
        "update" => {
            let (old_name, name) = args.split_once(' ').unwrap();
            Command::Update(UpdateArgs::new(
                old_name.to_string(),
                name.to_string(),
                DomainTypeRaw::ShadowBlockDomain,
            ))
        }
        "help" => Command::Help,
        _ => {
            println!("Invalid command");
            Command::Help
        }
    };
    cmd.run()
}
