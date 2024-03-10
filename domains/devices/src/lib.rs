#![no_std]

extern crate alloc;
extern crate malloc;
mod prob;

use crate::prob::Probe;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::cmp::min;
use core::ops::Range;
use fdt::Fdt;
use interface::{Basic, DeviceInfo, DevicesDomain};
use libsyscall::println;
use rref::{RRef, RRefVec};

#[derive(Debug, Clone)]
pub struct SystemDeviceInfo {
    pub address_range: Range<usize>,
    pub irq: Option<u32>,
    pub compatible: Option<String>,
}
#[derive(Debug, Clone)]
pub enum SystemDeviceType {
    PLIC(SystemDeviceInfo),
    VirtIoMMioBlock(SystemDeviceInfo),
    Uart(SystemDeviceInfo),
    VirtIoMMioNet(SystemDeviceInfo),
    Rtc(SystemDeviceInfo),
    VirtIoMMioInput(SystemDeviceInfo),
    VirtIoMMioGpu(SystemDeviceInfo),
}

impl AsRef<str> for SystemDeviceType {
    fn as_ref(&self) -> &str {
        match self {
            SystemDeviceType::PLIC(_) => "plic",
            SystemDeviceType::VirtIoMMioBlock(_) => "virtio-mmio-block",
            SystemDeviceType::Uart(_) => "uart",
            SystemDeviceType::VirtIoMMioNet(_) => "virtio-mmio-net",
            SystemDeviceType::Rtc(_) => "rtc",
            SystemDeviceType::VirtIoMMioInput(_) => "virtio-mmio-input",
            SystemDeviceType::VirtIoMMioGpu(_) => "virtio-mmio-gpu",
        }
    }
}

#[derive(Debug)]
pub struct DevicesDomainImpl {
    devices: Vec<SystemDeviceType>,
}

impl DevicesDomainImpl {
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
        }
    }

    pub fn push(&mut self, device_type: SystemDeviceType) {
        self.devices.push(device_type)
    }

    pub fn devices_count(&self) -> usize {
        self.devices.len()
    }
}

impl Basic for DevicesDomainImpl {}

impl DevicesDomain for DevicesDomainImpl {
    fn get_device(
        &self,
        name: RRefVec<u8>,
        mut info: RRef<DeviceInfo>,
    ) -> Option<RRef<DeviceInfo>> {
        let name = core::str::from_utf8(name.as_slice()).unwrap();

        libsyscall::println!("get_device: {}", name);

        let ty = self.devices.iter().find(|t| t.as_ref() == name)?;
        let t = match ty {
            SystemDeviceType::PLIC(t) => t.clone(),
            SystemDeviceType::VirtIoMMioBlock(t) => t.clone(),
            SystemDeviceType::Uart(t) => t.clone(),
            SystemDeviceType::VirtIoMMioNet(t) => t.clone(),
            SystemDeviceType::Rtc(t) => t.clone(),
            SystemDeviceType::VirtIoMMioInput(t) => t.clone(),
            SystemDeviceType::VirtIoMMioGpu(t) => t.clone(),
        };
        info.address_range = t.address_range;
        *info.irq = t.irq.unwrap_or(0);
        let slice = info.compatible.as_mut_slice();
        let compatible = t.compatible.clone().unwrap_or("".to_string());

        let copy_len = min(slice.len(), compatible.len());

        slice[..copy_len].copy_from_slice(compatible.as_bytes());

        Some(info)
    }
}

pub fn main() -> Arc<dyn DevicesDomain> {
    let dtb = libsyscall::get_dtb();
    let dtb = Fdt::new(dtb).unwrap();
    let mut domain = DevicesDomainImpl::new();
    dtb.probe_rtc().map(|ty| domain.push(ty));
    dtb.probe_uart().map(|ty| domain.push(ty));
    dtb.probe_plic().map(|ty| domain.push(ty));
    let virtio = dtb.probe_virtio();
    if let Some(virtio) = virtio {
        for ty in virtio {
            domain.push(ty);
        }
    }
    println!("{:#x?}", domain);
    println!("Probe {} devices.", domain.devices_count());
    Arc::new(domain)
}
