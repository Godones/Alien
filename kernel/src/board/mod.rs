use alloc::collections::BTreeMap;

use cfg_if::cfg_if;

use ksync::Mutex;

use crate::device::{DeviceInfo, DeviceType};

mod common;

pub static BOARD_DEVICES: Mutex<BTreeMap<DeviceType, DeviceInfo>> = Mutex::new(BTreeMap::new());

pub fn get_rtc_info() -> Option<(usize, usize)> {
    if let Some(rtc) = BOARD_DEVICES.lock().get(&DeviceType::Rtc) {
        return Some((rtc.base_addr, rtc.irq));
    }
    None
}

pub fn get_uart_info() -> Option<(usize, usize)> {
    if let Some(uart) = BOARD_DEVICES.lock().get(&DeviceType::Uart) {
        return Some((uart.base_addr, uart.irq));
    }
    None
}

pub fn get_gpu_info() -> Option<(usize, usize)> {
    if let Some(gpu) = BOARD_DEVICES.lock().get(&DeviceType::GPU) {
        return Some((gpu.base_addr, gpu.irq));
    }
    None
}

pub fn get_keyboard_info() -> Option<(usize, usize)> {
    if let Some(keyboard) = BOARD_DEVICES.lock().get(&DeviceType::KeyBoardInput) {
        return Some((keyboard.base_addr, keyboard.irq));
    }
    None
}

pub fn get_mouse_info() -> Option<(usize, usize)> {
    if let Some(mouse) = BOARD_DEVICES.lock().get(&DeviceType::MouseInput) {
        return Some((mouse.base_addr, mouse.irq));
    }
    None
}

pub fn get_block_device_info() -> Option<(usize, usize)> {
    if let Some(block) = BOARD_DEVICES.lock().get(&DeviceType::Block) {
        return Some((block.base_addr, block.irq));
    }
    None
}

pub fn get_net_device_info() -> Option<(usize, usize)> {
    if let Some(net) = BOARD_DEVICES.lock().get(&DeviceType::Network) {
        return Some((net.base_addr, net.irq));
    }
    None
}

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
