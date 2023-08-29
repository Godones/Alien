use alloc::collections::BTreeMap;
use alloc::sync::Arc;

use cfg_if::cfg_if;
use lazy_static::lazy_static;
use spin::Once;

pub use ext_interrupt::external_interrupt_handler;
use kernel_sync::Mutex;
use plic::{Mode, PLIC};

use crate::arch::hart_id;
use crate::config::CPU_NUM;
use crate::MACHINE_INFO;

mod ext_interrupt;
pub mod record;
mod timer;

pub static PLIC: Once<PLIC<CPU_NUM>> = Once::new();

lazy_static! {
    pub static ref DEVICE_TABLE: Mutex<BTreeMap<usize, Arc<dyn DeviceBase>>> =
        Mutex::new(BTreeMap::new());
}

pub trait DeviceBase: Sync + Send {
    fn hand_irq(&self);
}

pub fn init_plic() {
    let machine = MACHINE_INFO.get().unwrap();
    let addr = machine.plic.start;

    cfg_if! {
        if #[cfg(feature = "qemu")]{
            let privileges = [2;CPU_NUM];
            let plic = PLIC::new(addr, privileges);
            PLIC.call_once(|| plic);
            println!("init qemu plic success");
        }else if #[cfg(any(feature = "vf2", feature = "hifive"))]{
            let mut privileges = [2u8;CPU_NUM];
            // core 0 don't have S mode
            privileges[0] = 1;
            println!("PLIC context: {:?}",privileges);
            let plic = PLIC::new(addr, privileges);
            PLIC.call_once(|| plic);
            println!("init hifive or vf2 plic success");
        }
    }
}

/// Register a device to PLIC.
pub fn register_device_to_plic(irq: usize, device: Arc<dyn DeviceBase>) {
    let mut table = DEVICE_TABLE.lock();
    table.insert(irq, device);
    let hard_id = hart_id();
    println!(
        "plic enable irq {} for hart {}, priority {}",
        irq, hard_id, 1
    );
    let plic = PLIC.get().unwrap();
    plic.set_threshold(hard_id as u32, Mode::Machine, 1);
    plic.set_threshold(hard_id as u32, Mode::Supervisor, 0);
    plic.complete(hard_id as u32, Mode::Supervisor, irq as u32);
    plic.set_priority(irq as u32, 1);
    plic.enable(hard_id as u32, Mode::Supervisor, irq as u32);
}
