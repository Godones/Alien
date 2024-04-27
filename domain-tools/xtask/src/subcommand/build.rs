use std::{collections::BTreeMap, fs, path::Path};

use serde::Deserialize;

pub fn build_single(name: &str, log: &str) {
    let domain_list = fs::read_to_string("./domain-list.toml").unwrap();
    let config: Config = toml::from_str(&domain_list).unwrap();
    let all_members = config.domains.get("members").unwrap();
    let r_name = name.split_at(1).1;
    if !all_members.contains(&r_name.to_string()) {
        println!(
            "Domain [{}] is not in the members list, skip building",
            r_name
        );
        return;
    }
    let init_members = config.domains.get("init_members").unwrap();
    if init_members.contains(&r_name.to_string()) {
        build_domain(&name, log, "init");
    } else {
        let disk_members = config.domains.get("disk_members").unwrap();
        if disk_members.contains(&r_name.to_string()) {
            build_domain(&name, log, "disk");
        } else {
            println!(
                "Domain [{}] is not in the init or disk members list, skip building",
                r_name
            );
        }
    }
}

pub fn build_domain(name: &str, log: &str, dir: &str) {
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
        .arg(format!("./build/{}/{}", dir, name))
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
    std::process::Command::new("mkdir")
        .arg("./build/init")
        .status()
        .expect("failed to execute mkdir");
    println!("Start building all domains");
    let all_members = config.domains.get("members").unwrap();
    let init_members = config.domains.get("init_members").unwrap();
    for domain_name in init_members {
        if !all_members.contains(domain_name) {
            println!(
                "Domain [{}] is not in the members list, skip building",
                domain_name
            );
            continue;
        }
        let build_name = format!("g{domain_name}");
        build_domain(&build_name, log, "init");
    }
    let disk_members = config.domains.get("disk_members").unwrap();
    if !disk_members.is_empty() {
        std::process::Command::new("mkdir")
            .arg("./build/disk")
            .status()
            .expect("failed to execute mkdir");
        for domain_name in disk_members {
            if !all_members.contains(domain_name) {
                println!(
                    "Domain [{}] is not in the members list, skip building",
                    domain_name
                );
                continue;
            }
            let build_name = format!("g{domain_name}");
            build_domain(&build_name, log, "disk");
        }
    }
}
