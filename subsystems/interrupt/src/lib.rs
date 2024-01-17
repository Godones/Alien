#![no_std]
extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use spin::Once;

pub use ext_interrupt::external_interrupt_handler;
use ksync::Mutex;
use plic::{Mode, PLIC};

use arch::hart_id;
use config::CPU_NUM;
use device_interface::DeviceBase;
use platform::println;

mod ext_interrupt;
pub mod record;

pub static PLIC: Once<PLIC<CPU_NUM>> = Once::new();

pub static DEVICE_TABLE: Mutex<BTreeMap<usize, Arc<dyn DeviceBase>>> = Mutex::new(BTreeMap::new());

pub fn init_plic(plic_addr: usize) {
    #[cfg(feature = "qemu")]
    {
        let privileges = [2; CPU_NUM];
        let plic = PLIC::new(plic_addr, privileges);
        PLIC.call_once(|| plic);
        println!("Init qemu plic success");
    }
    #[cfg(any(feature = "vf2", feature = "hifive"))]
    {
        let mut privileges = [2; CPU_NUM];
        // core 0 don't have S mode
        privileges[0] = 1;
        println!("PLIC context: {:?}", privileges);
        let plic = PLIC::new(plic_addr, privileges);
        PLIC.call_once(|| plic);
        println!("Init hifive or vf2 plic success");
    }
}

/// Register a device to PLIC.
pub fn register_device_to_plic(irq: usize, device: Arc<dyn DeviceBase>) {
    let mut table = DEVICE_TABLE.lock();
    table.insert(irq, device);
    let hard_id = hart_id();
    println!(
        "PLIC enable irq {} for hart {}, priority {}",
        irq, hard_id, 1
    );
    let plic = PLIC.get().unwrap();
    plic.set_threshold(hard_id as u32, Mode::Machine, 1);
    plic.set_threshold(hard_id as u32, Mode::Supervisor, 0);
    plic.complete(hard_id as u32, Mode::Supervisor, irq as u32);
    plic.set_priority(irq as u32, 1);
    plic.enable(hard_id as u32, Mode::Supervisor, irq as u32);
}
