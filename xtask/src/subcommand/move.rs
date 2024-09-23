use std::{fs, path::Path};

use crate::subcommand::Config;

static DOMAIN_SET: [&str; 3] = ["common", "fs", "drivers"];

pub fn remove_to_space() {
    let domain_list = fs::read_to_string("../../../domains/domain-list.toml").unwrap();
    let config: Config = toml::from_str(&domain_list).unwrap();
    let all_members = config.domains.get("members").unwrap();
    for domain in all_members {
        let path = format!("./{}", domain);
        let path = Path::new(&path);
        if !path.exists() {
            // create a new domain project
            fs::create_dir_all(path).expect("failed to create domain directory");
            // create root cargo.toml which contains the members
            let cargo_toml = format!(
                "[workspace]\nmembers = [\n\t\"{}\",\n\t\"{}\",\n]\n\nresolver = \"2\"",
                domain,
                format!("g{}", domain)
            );
            fs::write(path.join("Cargo.toml"), cargo_toml).expect("failed to create Cargo.toml");
            // move the domain project to the new directory
            // find the domain project in directory which name is prefix of one of the DOMAIN_SET
            for ty in DOMAIN_SET.iter() {
                let domain_name = format!("{}/{}", ty, domain);
                let domain_path = Path::new(&domain_name);
                if domain_path.exists() {
                    // copy the domain project to the new directory
                    fs_extra::dir::copy(
                        domain_path,
                        path,
                        &fs_extra::dir::CopyOptions::new()
                            .copy_inside(true)
                            .overwrite(true),
                    )
                    .expect("failed to copy domain project");
                }
            }
            // copy the bin domain project to the new directory
            let domain_name = format!("generated/g{}", domain);
            let domain_path = Path::new(&domain_name);
            if domain_path.exists() {
                // copy the domain project to the new directory
                fs_extra::dir::copy(
                    domain_path,
                    path,
                    &fs_extra::dir::CopyOptions::new()
                        .copy_inside(true)
                        .overwrite(true),
                )
                .expect("failed to copy domain project");
            }
        }
    }
}
