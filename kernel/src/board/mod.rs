use cfg_if::cfg_if;

mod common;

cfg_if! {
    if #[cfg(feature="qemu")]{
        mod qemu;
        pub use qemu::*;
    }else if #[cfg(feature="cv1811")]{
        mod cv1811;
        pub use cv1811::*;
    }else if #[cfg(feature="hifive")]{
        mod unmatched;
        pub use unmatched::*;
    }else if #[cfg(feature="vf2")]{
        mod vf2;
        pub use vf2::*;
    }
}

cfg_if! {
    if #[cfg(any(feature = "vf2", feature = "hifive"))]{
       core::arch::global_asm!(r#"
            .section .data
            .global img_start
            .global img_end
            .align 12
            img_start:
                .incbin "./tools/sdcard.img"
            img_end:
        "#);
        extern "C" {
            pub fn img_start();
            pub fn img_end();
        }
        pub fn checkout_fs_img() {
            let img_start = img_start as usize;
            let img_end = img_end as usize;
            let img_size = img_end - img_start;
            println!(
                "img_start: {:#x}, img_end: {:#x}, img_size: {:#x}",
                img_start, img_end, img_size
            );
        }
    }
}
