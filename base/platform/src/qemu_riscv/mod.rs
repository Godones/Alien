pub mod config;

use spin::Once;

use crate::PlatformInfo;

pub static DTB: Once<usize> = Once::new();

pub fn init_dtb(dtb: Option<usize>) {
    let dtb_ptr = dtb.expect("No dtb found");
    DTB.call_once(|| dtb_ptr);
}

pub fn basic_machine_info() -> PlatformInfo {
    crate::common_riscv::basic::machine_info_from_dtb(*DTB.get().unwrap())
}

pub fn set_timer(time: usize) {
    crate::common_riscv::sbi::set_timer(time);
}

pub fn system_shutdown() -> ! {
    println!("shutdown...");
    crate::common_riscv::sbi::system_shutdown();
}

pub fn console_putchar(ch: u8) {
    crate::common_riscv::sbi::console_putchar(ch);
}
