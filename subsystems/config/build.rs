use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed={}", "src/lib.rs");
    let cpus = option_env!("SMP").unwrap_or("1");
    let cpus = cpus.parse::<usize>().unwrap();
    let config_file = Path::new("src/lib.rs");
    let config = fs::read_to_string(config_file).unwrap();
    let cpus = format!("pub const CPU_NUM: usize = {};\n", cpus);
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
