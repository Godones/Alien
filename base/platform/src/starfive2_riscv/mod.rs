pub mod config;
use core::ops::Range;

use spin::Once;

use crate::PlatformInfo;

pub const FDT: &[u8] = include_bytes!("../../../../tools/jh7110-visionfive-v2.dtb");

pub static DTB: Once<usize> = Once::new();

pub fn init_dtb(_dtb: Option<usize>) {
    let dtb_ptr = FDT.as_ptr() as usize;
    DTB.call_once(|| dtb_ptr);
}

pub fn basic_machine_info() -> PlatformInfo {
    let mut info = crate::common_riscv::basic::machine_info_from_dtb(*DTB.get().unwrap());
    info.initrd = Some(Range {
        start: INITRD.as_ptr() as usize,
        end: INITRD.as_ptr() as usize + INITRD.len(),
    });
    info
}

static INITRD: &'static [u8] = include_bytes!("../../../../build/initramfs.cpio.gz");

pub fn set_timer(time: usize) {
    crate::common_riscv::sbi::set_timer(time);
}

pub fn system_shutdown() -> ! {
    crate::common_riscv::sbi::system_shutdown();
}

pub fn console_putchar(ch: u8) {
    crate::common_riscv::sbi::console_putchar(ch);
}

#[allow(unused)]
pub fn console_getchar() -> char {
    crate::common_riscv::sbi::console_getchar()
}
