use std::{env, fs, fs::File, io::Write, path::Path};

fn main() {
    let outdir = env::var("OUT_DIR").unwrap();
    let link_script = Path::new(&outdir).join("link.lds");
    let mut script = File::create(&link_script).unwrap();
    let ld_path = Path::new("../tools/link.ld");
    let ld = fs::read_to_string(ld_path).unwrap();
    script.write_all(ld.as_bytes()).unwrap();
    println!("cargo:rustc-link-arg=-T{}", &link_script.display());
}
