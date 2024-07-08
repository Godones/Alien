fn main() {
    let platform = option_env!("PLATFORM").unwrap_or("qemu_riscv");
    println!("cargo::rustc-cfg={}", platform);
}
