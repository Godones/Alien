use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    // 指定target
    let outdir = env::var("OUT_DIR").unwrap();
    let link_script = Path::new(&outdir).join("link.lds");
    let mut script = File::create(&link_script).unwrap();
    script.write_all(include_bytes!("../tools/linker.ld")).unwrap();
    println!("cargo:rustc-link-arg=-T{}", &link_script.display());
}