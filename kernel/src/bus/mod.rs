#![allow(unused)]
use alloc::{string::String, vec};
use core::ops::Range;

use ::fdt::Fdt;
use constants::AlienResult;
use mem::PhysAddr;

use crate::bus::fdt::Probe;

mod fdt;
pub mod mmio;
pub mod pci;
pub mod platform;

#[derive(Debug, Clone)]
pub struct CommonDeviceInfo {
    pub address_range: Range<PhysAddr>,
    pub irq: Option<u32>,
    pub compatible: Option<String>,
}
#[derive(Debug, Clone)]
pub enum CommonDeviceType {
    PLIC(CommonDeviceInfo),
    Uart(CommonDeviceInfo),
    Rtc(CommonDeviceInfo),
    VirtIo(CommonDeviceInfo),
    Pci(CommonDeviceInfo),
}

pub fn init_with_dtb() -> AlienResult<()> {
    let ptr = ::platform::platform_dtb_ptr();
    let dtb = unsafe { Fdt::from_ptr(ptr as *const u8) }.unwrap();

    let mut devices = vec![];
    dtb.probe_rtc().map(|ty| {
        devices.push(ty);
    });
    dtb.probe_uart().map(|ty| {
        devices.push(ty);
    });
    dtb.probe_plic().map(|ty| {
        devices.push(ty);
    });
    dtb.probe_pci().map(|ty| {
        devices.push(ty);
    });
    let virtio = dtb.probe_virtio();
    if let Some(virtio) = virtio {
        for ty in virtio {
            devices.push(ty);
        }
    }
    devices.into_iter().for_each(|ty| match ty {
        CommonDeviceType::PLIC(info) => platform::register_platform_device(info, "plic"),
        CommonDeviceType::Uart(info) => platform::register_platform_device(info, "uart"),
        CommonDeviceType::Rtc(info) => platform::register_platform_device(info, "rtc"),
        CommonDeviceType::VirtIo(info) => mmio::register_mmio_device(info),
        CommonDeviceType::Pci(info) => pci::pci_init(info),
    });
    Ok(())
}
