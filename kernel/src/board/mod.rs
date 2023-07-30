#[cfg(feature = "cv1811")]
pub use cv1811::*;
#[cfg(feature = "hifive")]
pub use unmatched::*;
#[cfg(feature = "vf2")]
pub use vf2::*;

#[cfg(feature = "cv1811")]
mod cv1811;
#[cfg(feature = "hifive")]
mod unmatched;
#[cfg(feature = "vf2")]
mod vf2;

#[cfg(any(feature = "vf2", feature = "cv1811h", feature = "hifive"))]
core::arch::global_asm!(
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

#[cfg(any(feature = "vf2", feature = "cv1811h", feature = "hifive"))]
extern "C" {
    pub fn img_start();
    pub fn img_end();
}

#[cfg(any(feature = "vf2", feature = "cv1811h", feature = "hifive"))]
pub fn checkout_fs_img() {
    let img_start = img_start as usize;
    let img_end = img_end as usize;
    let img_size = img_end - img_start;
    println!(
        "img_start: {:#x}, img_end: {:#x}, img_size: {:#x}",
        img_start, img_end, img_size
    );
}
