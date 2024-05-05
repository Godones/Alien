#![no_std]
extern crate alloc;

use alloc::{collections::BTreeMap, format, string::String, sync::Arc};

use arch::hart_id;
use config::CPU_NUM;
use device_interface::DeviceBase;
use ksync::Mutex;
use platform::println;
use plic::{Mode, PLIC};
use spin::Once;

pub static PLIC: Once<PLIC<CPU_NUM>> = Once::new();
pub static INTERRUPT_RECORD: Mutex<BTreeMap<usize, usize>> = Mutex::new(BTreeMap::new());
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

pub fn external_interrupt_handler() {
    let plic = PLIC.get().unwrap();
    let hart_id = hart_id();
    let irq = plic.claim(hart_id as u32, Mode::Supervisor);
    let table = DEVICE_TABLE.lock();
    let device = table
        .get(&(irq as usize))
        .or_else(|| panic!("no device for irq {}", irq))
        .unwrap();
    device.handle_irq();
    plic.complete(hart_id as u32, Mode::Supervisor, irq);
    let mut interrupts = INTERRUPT_RECORD.lock();
    let value = interrupts.entry(irq as usize).or_insert(0);
    *value += 1;
}

/// Serializes the number of interrupts
///
/// # Return
/// irq{}: number
pub fn interrupts_info() -> String {
    let interrupts = INTERRUPT_RECORD.lock();
    let mut res = String::new();
    interrupts.iter().for_each(|(irq, value)| {
        res.push_str(&format!("{}: {}\r\n", irq, value));
    });
    res
}

/// Increase the number of interrupts
pub fn record_irq(irq: usize) {
    let mut interrupts = INTERRUPT_RECORD.lock();
    let value = interrupts.entry(irq).or_insert(0);
    *value += 1;
}
