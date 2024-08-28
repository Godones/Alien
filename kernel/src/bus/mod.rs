#![allow(unused)]
use alloc::{string::String, vec};
use core::ops::Range;

use ::fdt::Fdt;
use mem::PhysAddr;

use crate::{bus::fdt::Probe, error::AlienResult};

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
    Plic(CommonDeviceInfo),
    Uart(CommonDeviceInfo),
    Rtc(CommonDeviceInfo),
    VirtIo(CommonDeviceInfo),
    Pci(CommonDeviceInfo),
    Ramdisk(CommonDeviceInfo),
    LoopBack(CommonDeviceInfo),
    SdCard(CommonDeviceInfo),
}

pub fn init_with_dtb() -> AlienResult<()> {
    let ptr = ::platform::platform_dtb_ptr();
    let dtb = unsafe { Fdt::from_ptr(ptr as *const u8) }.unwrap();

    let mut devices = vec![];
    if let Some(ty) = dtb.probe_rtc() {
        devices.push(ty);
    }
    if let Some(ty) = dtb.probe_uart() {
        devices.push(ty);
    }
    if let Some(ty) = dtb.probe_plic() {
        devices.push(ty);
    }
    if let Some(ty) = dtb.probe_pci() {
        devices.push(ty);
    }
    let virtio = dtb.probe_virtio();
    if let Some(virtio) = virtio {
        for ty in virtio {
            devices.push(ty);
        }
    }

    #[cfg(feature = "bench")]
    {
        let ramdisk_start = RAMDISK.as_ptr() as usize;
        let len = RAMDISK.len();
        let info = CommonDeviceInfo {
            address_range: PhysAddr::from(ramdisk_start)..PhysAddr::from(ramdisk_start + len),
            irq: None,
            compatible: None,
        };
        devices.push(CommonDeviceType::Ramdisk(info));
    }

    #[cfg(vf2)]
    {
        #[cfg(not(vf2_sd))]
        {
            let ramdisk_start = RAMDISK.as_ptr() as usize;
            let len = RAMDISK.len();
            let info = CommonDeviceInfo {
                address_range: PhysAddr::from(ramdisk_start)..PhysAddr::from(ramdisk_start + len),
                irq: None,
                compatible: None,
            };
            devices.push(CommonDeviceType::Ramdisk(info));
        }

        let fake_nic = CommonDeviceInfo {
            address_range: PhysAddr::from(0)..PhysAddr::from(0 + 0),
            irq: Some(0),
            compatible: None,
        };
        devices.push(CommonDeviceType::LoopBack(fake_nic));
    }
    #[cfg(all(vf2, vf2_sd))]
    dtb.probe_sd().map(|ty| {
        devices.push(ty);
    });

    devices.into_iter().for_each(|ty| match ty {
        CommonDeviceType::Plic(info) => platform::register_platform_device(info, "plic"),
        CommonDeviceType::Uart(info) => platform::register_platform_device(info, "uart"),
        CommonDeviceType::Rtc(info) => platform::register_platform_device(info, "rtc"),
        CommonDeviceType::VirtIo(info) => mmio::register_mmio_device(info),
        CommonDeviceType::Pci(info) => pci::pci_init(info),
        CommonDeviceType::Ramdisk(info) => platform::register_platform_device(info, "ramdisk"),
        CommonDeviceType::LoopBack(info) => platform::register_platform_device(info, "loopback"),
        CommonDeviceType::SdCard(info) => platform::register_platform_device(info, "sdcard"),
    });
    Ok(())
}

#[cfg(feature = "bench")]
static RAMDISK: &'static [u8] = &[0u8; 1024];
#[cfg(all(vf2, not(vf2_sd)))]
static RAMDISK: &'static [u8] = include_bytes!("../../../build/sdcard.img");
