use std::fs;

use crate::subcommand::{Config, DOMAIN_SET};

pub fn fmt_domain(name: String) {
    let domain_list = fs::read_to_string("./domains/domain-list.toml").unwrap();
    let config: Config = toml::from_str(&domain_list).unwrap();
    if name.is_empty() {
        // format all domain projects
        let all_members = config.domains.get("members").unwrap();
        for domain_name in all_members {
            fmt_one_domain(domain_name);
        }
        return;
    } else {
        fmt_one_domain(&name);
    }
}

fn fmt_one_domain(name: &String) {
    for prefix in DOMAIN_SET {
        let path = format!("./domains/{}/{}", prefix, name);
        let dir = std::path::Path::new(&path);
        if dir.exists() {
            println!("Formatting domain project {}", path);
            let _cmd = std::process::Command::new("cargo")
                .arg("fmt")
                .current_dir(path)
                .status()
                .expect("failed to format domain project");
            return;
        }
    }
}
