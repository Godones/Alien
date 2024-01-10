mod probe;

use alloc::collections::BTreeMap;

use cfg_if::cfg_if;

use crate::ksync::Mutex;

use crate::device::DeviceType;

pub static BOARD_DEVICES: Mutex<BTreeMap<DeviceType, DeviceInfo>> = Mutex::new(BTreeMap::new());

pub fn get_rtc_info() -> Option<DeviceInfo> {
    BOARD_DEVICES
        .lock()
        .get(&DeviceType::Rtc)
        .map(|rtc| rtc.clone())
}

pub fn get_uart_info() -> Option<DeviceInfo> {
    BOARD_DEVICES
        .lock()
        .get(&DeviceType::Uart)
        .map(|uart| uart.clone())
}

pub fn get_gpu_info() -> Option<DeviceInfo> {
    BOARD_DEVICES
        .lock()
        .get(&DeviceType::GPU)
        .map(|gpu| gpu.clone())
}

pub fn get_keyboard_info() -> Option<DeviceInfo> {
    BOARD_DEVICES
        .lock()
        .get(&DeviceType::KeyBoardInput)
        .map(|keyboard| keyboard.clone())
}

pub fn get_mouse_info() -> Option<DeviceInfo> {
    BOARD_DEVICES
        .lock()
        .get(&DeviceType::MouseInput)
        .map(|mouse| mouse.clone())
}

pub fn get_block_device_info() -> Option<DeviceInfo> {
    BOARD_DEVICES
        .lock()
        .get(&DeviceType::Block)
        .map(|block| block.clone())
}

pub fn get_net_device_info() -> Option<DeviceInfo> {
    BOARD_DEVICES
        .lock()
        .get(&DeviceType::Network)
        .map(|net| net.clone())
}

use crate::board::probe::DeviceInfo;
cfg_if! {
    if #[cfg(feature="qemu")]{
        mod qemu;
        pub use qemu::*;
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
