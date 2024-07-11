use std::{env, fs, fs::File, io::Write, path::Path};

fn main() {
    let outdir = env::var("OUT_DIR").unwrap();
    let link_script = Path::new(&outdir).join("link.lds");
    let mut script = File::create(&link_script).unwrap();
    let ld_path = Path::new("../tools/link.ld");
    let ld = fs::read_to_string(ld_path).unwrap();
    let platform = option_env!("PLATFORM").unwrap_or("qemu_riscv");
    if platform == "vf2" {
        let base_addr = 0x40200000;
        let base_addr = format!("BASE_ADDRESS = {};", base_addr);
        let mut new_config = String::new();
        for line in ld.lines() {
            if line.starts_with("BASE_ADDRESS = ") {
                new_config.push_str(base_addr.as_str());
            } else {
                new_config.push_str(line);
                new_config.push_str("\n");
            }
        }
        script.write_all(new_config.as_bytes()).unwrap();
    }else {
        script.write_all(ld.as_bytes()).unwrap();
    }
    println!("cargo:rustc-link-arg=-T{}", &link_script.display());
    println!("cargo::rustc-cfg={}", platform);
}
