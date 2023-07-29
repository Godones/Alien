use std::{env, fs};
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    // 指定target
    let outdir = env::var("OUT_DIR").unwrap();
    let link_script = Path::new(&outdir).join("link.lds");
    let mut script = File::create(&link_script).unwrap();

    let ld_path = Path::new("../tools/link.ld");
    let ld = fs::read_to_string(ld_path).unwrap();

    #[cfg(not(feature = "vf2"))]
        let base_addr = 0x80200000usize;
    #[cfg(feature = "vf2")]
        let base_addr: usize = 0x80200000;
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
    println!("cargo:rustc-link-arg=-T{}", &link_script.display());
}
