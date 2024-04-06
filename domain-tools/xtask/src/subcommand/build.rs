use std::path::Path;

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
        .arg(format!("./build/{}_domain.bin", name))
        .status()
        .expect("failed to execute cp");
    println!("Copy domain [{}] project success", name)
}
