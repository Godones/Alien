use std::{fs, path::Path};

use crate::subcommand::{Config, DOMAIN_SET};

pub fn clean_domain(name: String) {
    let domain_list = fs::read_to_string("./domains/domain-list.toml").unwrap();
    let config: Config = toml::from_str(&domain_list).unwrap();
    if name.is_empty() {
        // clean all domain projects
        let all_members = config.domains.get("members").unwrap();
        for domain_name in all_members {
            clean_one_domain(domain_name);
        }
        println!("Cleaning ELF");
        std::process::Command::new("rm")
            .arg("-rf")
            .arg("./domains/target")
            .status()
            .expect("failed to clean domain project");
    } else {
        clean_one_domain(&name);
    }
}

fn clean_one_domain(name: &String) {
    for prefix in DOMAIN_SET {
        let path = format!("./domains/{}/{}/target", prefix, name);
        let dir = Path::new(&path);
        if dir.exists() {
            println!("Cleaning domain project {}", path);
            std::process::Command::new("rm")
                .arg("-dr")
                .arg(path)
                .status()
                .expect("failed to clean domain project");
            return;
        }
    }
}
