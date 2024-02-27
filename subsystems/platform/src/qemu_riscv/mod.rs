pub mod config;
use crate::PlatformInfo;
use spin::Once;

pub static DTB: Once<usize> = Once::new();

pub fn init_dtb(dtb: Option<usize>) {
    let dtb_ptr = dtb.expect("No dtb found");
    DTB.call_once(|| dtb_ptr);
}

pub fn basic_machine_info() -> PlatformInfo {
    crate::common_riscv::basic::machine_info_from_dtb(*DTB.get().unwrap())
}

/// 设置定时器
pub fn set_timer(time: usize) {
    crate::common_riscv::sbi::set_timer(time);
}

pub fn system_shutdown() -> ! {
    crate::common_riscv::sbi::system_shutdown();
}

/// Warp sbi SBI_CONSOLE_PUT_CHAR  call
pub fn console_putchar(ch: u8) {
    crate::common_riscv::sbi::console_putchar(ch);
}

/// Warp sbi SBI_CONSOLE_GET_CHAR  call
#[allow(unused)]
pub fn console_getchar() -> char {
    crate::common_riscv::sbi::console_getchar()
}
