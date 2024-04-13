use std::{collections::BTreeMap, fs, path::Path};

use serde::Deserialize;
pub fn build_domain(name: &str, log: &str) {
    // change the directory to the domain project
    // run cargo build
    let generated_path = Path::new("./domains/generated");
    println!("Changed directory to {}", generated_path.display());
    let _cmd = std::process::Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("-p")
        .arg(name)
        .env("LOG", log)
        .current_dir(generated_path)
        .status()
        .expect("failed to execute cargo build");
    println!("Build domain [{}] project success", name);
    std::process::Command::new("cp")
        .arg(format!(
            "./target/riscv64gc-unknown-none-elf/release/{}",
            name
        ))
        .arg(format!("./build/{}", name))
        .status()
        .expect("failed to execute cp");
    println!("Copy domain [{}] project success", name)
}

#[derive(Deserialize)]
struct Config {
    domains: BTreeMap<String, Vec<String>>,
}

pub fn build_all(log: &str) {
    let domain_list = fs::read_to_string("./domain-list.toml").unwrap();
    let config: Config = toml::from_str(&domain_list).unwrap();
    println!("Start building all domains");
    let list = config.domains.get("members").unwrap();
    for domain_name in list {
        let build_name = format!("g{domain_name}");
        build_domain(&build_name, log);
    }
}
