use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::string::String;
use std::{format, fs};

fn main() {
    println!("cargo:rerun-if-changed={}", "src/");
    let path = Path::new("src/trace/kernel_symbol.S");
    if !path.exists() {
        let mut file = File::create(path).unwrap();
        write!(file, ".section .rodata\n").unwrap();
        write!(file, ".align 3\n").unwrap();
        write!(file, ".global symbol_num\n").unwrap();
        write!(file, ".global symbol_address\n").unwrap();
        write!(file, ".global symbol_index\n").unwrap();
        write!(file, ".global symbol_name\n").unwrap();
        write!(file, "symbol_num:\n").unwrap();
        write!(file, ".quad {}\n", 0).unwrap();
        write!(file, "symbol_address:\n").unwrap();
        write!(file, "symbol_index:\n").unwrap();
        write!(file, "symbol_name:\n").unwrap();
    }
    rewrite_config();
}

pub fn rewrite_config() {
    let cpus = option_env!("SMP").unwrap_or("1");
    let cpus = cpus.parse::<usize>().unwrap();
    let config_file = Path::new("src/config.rs");
    let config = fs::read_to_string(config_file).unwrap();
    let cpus = format!("pub const CPU_NUM: usize = {};\n", cpus);
    // let regex = regex::Regex::new(r"pub const CPU_NUM: usize = \d+;").unwrap();
    // config = regex.replace_all(&config, cpus.as_str()).to_string();
    let mut new_config = String::new();
    for line in config.lines() {
        if line.starts_with("pub const CPU_NUM: usize = ") {
            new_config.push_str(cpus.as_str());
        } else {
            new_config.push_str(line);
            new_config.push_str("\n");
        }
    }
    fs::write(config_file, new_config).unwrap();
}
