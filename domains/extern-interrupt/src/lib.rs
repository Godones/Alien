#![no_std]
#![deny(unsafe_code)]

extern crate alloc;
extern crate malloc;

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::sync::Arc;
use config::CPU_NUM;
use core::cmp::min;
use interface::{Basic, DeviceBase, PLICDomain};
use ksync::Mutex;
use libsyscall::DeviceType;
use plic::{Mode, PLIC};
use region::SafeIORegion;
use rref::{RRefVec, RpcResult};

#[derive(Debug)]
pub struct PLICDomainImpl<const H: usize> {
    plic: PLIC<H>,
    table: Arc<Mutex<BTreeMap<usize, Arc<dyn DeviceBase>>>>,
    count: Arc<Mutex<BTreeMap<usize, usize>>>,
}

impl<const H: usize> PLICDomainImpl<H> {
    pub fn new(region: SafeIORegion, privileges: [u8; H]) -> Self {
        Self {
            plic: PLIC::new(region, privileges),
            table: Arc::new(Mutex::new(BTreeMap::new())),
            count: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }
}

impl<const H: usize> Basic for PLICDomainImpl<H> {}

impl<const H: usize> PLICDomain for PLICDomainImpl<H> {
    fn handle_irq(&self) -> RpcResult<()> {
        let plic = &self.plic;
        let hart_id = arch::hart_id();
        let irq = plic.claim(hart_id as u32, Mode::Supervisor) as usize;
        let table = self.table.lock();
        let device = table
            .get(&irq)
            .or_else(|| panic!("no device for irq {}", irq))
            .unwrap();
        device.handle_irq()?;
        *self
            .count
            .lock()
            .get_mut(&irq)
            .or_else(|| panic!("no device for irq {}", irq))
            .unwrap() += 1;
        plic.complete(hart_id as u32, Mode::Supervisor, irq as u32);
        Ok(())
    }

    fn register_irq(&self, irq: usize, device: Arc<dyn DeviceBase>) -> RpcResult<()> {
        let hard_id = arch::hart_id();
        libsyscall::println!(
            "PLIC enable irq {} for hart {}, priority {}",
            irq,
            hard_id,
            1
        );
        let plic = &self.plic;
        plic.set_threshold(hard_id as u32, Mode::Machine, 1);
        plic.set_threshold(hard_id as u32, Mode::Supervisor, 0);
        plic.complete(hard_id as u32, Mode::Supervisor, irq as u32);
        plic.set_priority(irq as u32, 1);
        plic.enable(hard_id as u32, Mode::Supervisor, irq as u32);
        let mut table = self.table.lock();
        table.insert(irq, device);
        self.count.lock().insert(irq, 0);
        Ok(())
    }

    fn irq_info(&self, mut buf: RRefVec<u8>) -> RpcResult<RRefVec<u8>> {
        let interrupts = self.count.lock();
        let mut res = String::new();
        interrupts.iter().for_each(|(irq, value)| {
            res.push_str(&format!("{}: {}\r\n", irq, value));
        });
        let copy_len = min(buf.len(), res.as_bytes().len());
        buf.as_mut_slice()[..copy_len].copy_from_slice(&res.as_bytes()[..copy_len]);
        Ok(buf)
    }
}

pub fn main() -> Arc<dyn PLICDomain> {
    let plic_space = libsyscall::get_device_space(DeviceType::PLIC).unwrap();
    let plic_space = SafeIORegion::from(plic_space).unwrap();
    let privileges = [2; CPU_NUM];
    let domain_impl = PLICDomainImpl::<CPU_NUM>::new(plic_space, privileges);
    libsyscall::println!("Init qemu plic success");
    Arc::new(domain_impl)
}
