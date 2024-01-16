pub mod config;

use spin::Once;

pub const FDT: &[u8] = include_bytes!("../../../../tools/jh7110-visionfive-v2.dtb");

pub static DTB: Once<usize> = Once::new();

pub fn init_dtb(dtb: Option<usize>) {
    let dtb_ptr = dtb.unwrap_or(FDT.as_ptr() as usize);
    DTB.call_once(|| dtb_ptr);
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
