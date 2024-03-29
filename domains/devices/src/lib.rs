#![no_std]

extern crate alloc;
mod prob;

use alloc::{boxed::Box, string::String, vec::Vec};
use core::{cmp::min, ops::Range};

use basic::println;
use constants::AlienResult;
use fdt::Fdt;
use interface::{Basic, DeviceInfo, DevicesDomain};
use ksync::Mutex;
use rref::RRef;

use crate::prob::Probe;

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
            SystemDeviceType::VirtIoMMioNet(_) => "virtio-mmio-virtio-mmio-net",
            SystemDeviceType::Rtc(_) => "rtc",
            SystemDeviceType::VirtIoMMioInput(_) => "virtio-mmio-input",
            SystemDeviceType::VirtIoMMioGpu(_) => "virtio-mmio-gpu",
        }
    }
}

#[derive(Debug)]
pub struct DevicesDomainImpl {
    devices: Mutex<Vec<SystemDeviceType>>,
}

impl DevicesDomainImpl {
    pub fn new() -> Self {
        Self {
            devices: Mutex::new(Vec::new()),
        }
    }
}

impl Basic for DevicesDomainImpl {}

impl DevicesDomain for DevicesDomainImpl {
    fn init(&self, dtb: &'static [u8]) -> AlienResult<()> {
        let dtb = Fdt::new(dtb).unwrap();
        dtb.probe_rtc().map(|ty| self.devices.lock().push(ty));
        dtb.probe_uart().map(|ty| self.devices.lock().push(ty));
        dtb.probe_plic().map(|ty| self.devices.lock().push(ty));
        let virtio = dtb.probe_virtio();
        if let Some(virtio) = virtio {
            for ty in virtio {
                self.devices.lock().push(ty);
            }
        }
        // println!("{:#x?}", self);
        println!("Probe {} devices.", self.devices.lock().len());
        Ok(())
    }

    fn index_device(
        &self,
        index: usize,
        mut info: RRef<DeviceInfo>,
    ) -> AlienResult<RRef<DeviceInfo>> {
        let devices = self.devices.lock();
        let device = devices.get(index);
        match device {
            Some(ty) => {
                let t = match ty {
                    SystemDeviceType::PLIC(t) => t,
                    SystemDeviceType::VirtIoMMioBlock(t) => t,
                    SystemDeviceType::Uart(t) => t,
                    SystemDeviceType::VirtIoMMioNet(t) => t,
                    SystemDeviceType::Rtc(t) => t,
                    SystemDeviceType::VirtIoMMioInput(t) => t,
                    SystemDeviceType::VirtIoMMioGpu(t) => t,
                };
                info.address_range = t.address_range.clone();
                info.irq = t.irq.unwrap_or(0);
                info.next = index + 1;
                let name = ty.as_ref();
                let copy_len = min(info.name.len(), name.len());
                info.name[..copy_len].copy_from_slice(&name.as_bytes()[..copy_len]);
                Ok(info)
            }
            None => {
                info.next = 0;
                Ok(info)
            }
        }
    }
}

pub fn main() -> Box<dyn DevicesDomain> {
    let domain = DevicesDomainImpl::new();
    Box::new(domain)
}
