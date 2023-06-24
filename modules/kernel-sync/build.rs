use std::fs;
use std::path::Path;

fn main() {}


pub fn rewrite_config() {
    let cpus = option_env!("SMP").unwrap_or("4");
    let cpus = cpus.parse::<usize>().unwrap();
    let config_file = Path::new("src/interrupt.rs");
    let config = fs::read_to_string(config_file).unwrap();
    let cpus = format!("const MAX_CORE_NUM: usize = {};\n", cpus);
    // let regex = regex::Regex::new(r"pub const CPU_NUM: usize = \d+;").unwrap();
    // config = regex.replace_all(&config, cpus.as_str()).to_string();
    let mut new_config = String::new();
    for line in config.lines() {
        if line.starts_with("const MAX_CORE_NUM: usize = ") {
            new_config.push_str(cpus.as_str());
        } else {
            new_config.push_str(line);
            new_config.push_str("\n");
        }
    }
    fs::write(config_file, new_config).unwrap();
}