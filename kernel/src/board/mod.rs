use core::arch::global_asm;

#[cfg(feature = "cv1811")]
pub use cv1811::*;
#[cfg(feature = "vf2")]
pub use vf2::*;

mod cv1811;
mod vf2;


// pub static FAT32_IMG: &[u8] = include_bytes!("../../../tools/sdcard.img");
#[cfg(any(feature = "vf2", feature = "cv1811h"))]
global_asm!(
    r#"
    .section .data
    .global img_start
    .global img_end
    .align 12
    img_start:
        .incbin "./tools/sdcard.img"
    img_end:
    "#
);

#[cfg(any(feature = "vf2", feature = "cv1811h"))]
extern "C" {
    pub fn img_start();
    pub fn img_end();
}

pub fn checkout_fs_img() {
    let img_start = img_start as usize;
    let img_end = img_end as usize;
    let img_size = img_end - img_start;
    println!("img_start: {:#x}, img_end: {:#x}, img_size: {:#x}", img_start, img_end, img_size);
}