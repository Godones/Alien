use std::{
    io::{Read, Seek, Write},
    path::{Path, PathBuf},
};

use fs_extra::dir::CopyOptions;

#[derive(Debug, Copy, Clone)]
pub enum DomainType {
    Common,
    Fs,
    Driver,
}

impl DomainType {
    pub fn to_string(&self) -> String {
        match self {
            DomainType::Common => "common".to_string(),
            DomainType::Fs => "fs".to_string(),
            DomainType::Driver => "drivers".to_string(),
        }
    }
}

pub fn create_domain(name: &str) {
    let mut chose = String::new();
    println!("Please chose the domain type:");
    println!("1. Common");
    println!("2. Fs");
    println!("3. Driver");
    let ty = match std::io::stdin().read_line(&mut chose) {
        Ok(_) => match chose.as_str() {
            "1\n" => DomainType::Common,
            "2\n" => DomainType::Fs,
            "3\n" => DomainType::Driver,
            _ => {
                println!("Error: invalid input");
                return;
            }
        },
        Err(_) => {
            println!("Error: failed to read line");
            return;
        }
    };
    // input interface
    println!("Please input the domain interface name:");
    let mut interface = String::new();
    let interface = match std::io::stdin().read_line(&mut interface) {
        Ok(_) => interface,
        Err(_) => {
            println!("Error: failed to read line");
            return;
        }
    };
    println!("Creating new domain project: {}, type: {:?}", name, ty);
    println!("The domain interface: {}", interface.trim());

    create_lib_crate(&interface, name, ty);
    create_bin_crate(&interface, name, ty);
    println!("Success: create domain project {}", name);
}

fn create_lib_crate(interface_name: &str, domain_name: &str, ty: DomainType) {
    let path = PathBuf::from(format!("./domains/{}/{}", ty.to_string(), domain_name));
    if path.exists() {
        println!("Error: the domain project already exists");
        return;
    } else {
        std::fs::create_dir_all(&path).unwrap();
    }
    // copy lib-template dir
    let copy_options = CopyOptions::new().content_only(true);

    let template_path = Path::new("./domain-tools/xtask/lib-template");

    fs_extra::dir::copy(template_path, &path, &copy_options).unwrap();
    let lib_path = path.join("src/lib.rs");
    let mut lib = std::fs::OpenOptions::new()
        .write(true)
        .read(true)
        .open(lib_path)
        .unwrap();
    let mut content = String::new();
    lib.read_to_string(&mut content).unwrap();
    let new_content = content.replace("INTERFACE", interface_name.trim());
    lib.set_len(0).unwrap();
    lib.seek(std::io::SeekFrom::Start(0)).unwrap();
    lib.write_all(new_content.as_bytes()).unwrap();

    let dep_path = path.join("Cargo.toml");
    let mut dep = std::fs::OpenOptions::new()
        .write(true)
        .read(true)
        .open(dep_path)
        .unwrap();
    let mut content = String::new();
    dep.read_to_string(&mut content).unwrap();
    let new_content = content.replace("PACKAGE", domain_name);
    dep.set_len(0).unwrap();
    dep.seek(std::io::SeekFrom::Start(0)).unwrap();
    dep.write_all(new_content.as_bytes()).unwrap();
}

fn create_bin_crate(interface_name: &str, domain_name: &str, ty: DomainType) {
    let path = PathBuf::from(format!("./domains/generated/g{}", domain_name));
    if path.exists() {
        println!("Error: the domain project already exists");
        return;
    } else {
        std::fs::create_dir_all(&path).unwrap();
    }
    let template_path = Path::new("./domain-tools/xtask/bin-template");
    let copy_options = CopyOptions::new().content_only(true);
    fs_extra::dir::copy(template_path, &path, &copy_options).unwrap();

    let dep_path = path.join("Cargo.toml");
    let mut dep = std::fs::OpenOptions::new()
        .write(true)
        .read(true)
        .open(dep_path)
        .unwrap();
    let mut content = String::new();
    dep.read_to_string(&mut content).unwrap();

    let new_content = content
        .replace("PACKAGE", &format!("g{}", domain_name))
        .replace("DOMAIN_NAME", domain_name)
        .replace("TY", &ty.to_string());
    dep.set_len(0).unwrap();
    dep.seek(std::io::SeekFrom::Start(0)).unwrap();
    dep.write_all(new_content.as_bytes()).unwrap();

    let main_path = path.join("src/main.rs");
    let mut main = std::fs::OpenOptions::new()
        .write(true)
        .read(true)
        .open(main_path)
        .unwrap();
    let mut main_content = String::new();
    main.read_to_string(&mut main_content).unwrap();
    let new_main_content = main_content
        .replace("DOMAIN_NAME", domain_name)
        .replace("INTERFACE", interface_name.trim());
    main.set_len(0).unwrap();
    main.seek(std::io::SeekFrom::Start(0)).unwrap();
    main.write_all(new_main_content.as_bytes()).unwrap();
}
