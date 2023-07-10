#![no_std]

use core::cmp::min;
use core::fmt::Debug;
use core::ops::Range;

use fdt::Fdt;

const MEMORY: &str = "memory";
const SERIAL: &str = "serial";
const UART: &str = "uart";
const PLIC: &str = "plic";
const CLINT: &str = "clint";
const RTC: &str = "rtc";

#[derive(Clone)]
pub struct MachineInfo {
    pub model: [u8; 32],
    pub smp: usize,
    pub memory: Range<usize>,
    pub uart: [UartInfo; 8],
    pub plic: Range<usize>,
    pub clint: Range<usize>,
    pub rtc: RtcInfo,
    pub uart_count: usize,
}

impl Debug for MachineInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let model = core::str::from_utf8(&self.model).unwrap();
        f.debug_struct("MachineInfo")
            .field("model", &model)
            .field("smp", &self.smp)
            .field("memory", &self.memory)
            .field("uart", &&self.uart[..self.uart_count])
            .field("plic", &self.plic)
            .field("clint", &self.clint)
            .field("rtc", &self.rtc)
            .finish()
    }
}

#[derive(Debug, Default, Clone)]
pub struct RtcInfo {
    pub range: Range<usize>,
    pub irq: usize,
}

#[derive(Debug, Default, Copy, Clone)]
#[allow(unused)]
pub struct UartInfo {
    base: usize,
    irq: usize,
}

pub fn machine_info_from_dtb(ptr: usize) -> MachineInfo {
    let fdt = unsafe { Fdt::from_ptr(ptr as *const u8).unwrap() };
    walk_dt(fdt)
}

fn walk_dt(fdt: Fdt) -> MachineInfo {
    let mut machine = MachineInfo {
        model: [0; 32],
        smp: 0,
        memory: 0..0,
        uart: [UartInfo::default(); 8],
        plic: 0..0,
        clint: 0..0,
        rtc: RtcInfo::default(),
        uart_count: 0,
    };
    let x = fdt.root();
    machine.smp = fdt.cpus().count();
    let model = x.model().as_bytes();
    let len = min(model.len(), machine.model.len());
    machine.model[0..len].copy_from_slice(&model[..len]);

    let mut uart_count = 0;

    for node in fdt.all_nodes() {
        if node.name.starts_with(MEMORY) {
            let reg = node.reg().unwrap();
            reg.for_each(|x| {
                machine.memory = Range {
                    start: x.starting_address as usize,
                    end: x.starting_address as usize + x.size.unwrap(),
                }
            })
        } else if node.name.starts_with(SERIAL) || node.name.starts_with(UART) {
            let reg = node.reg();
            if reg.is_none() {
                continue;
            }
            let reg = reg.unwrap();
            let val = node.interrupts().unwrap().next().unwrap();
            // let irq = node.property("interrupts").unwrap().value;
            // let irq = u32::from_be_bytes(irq.try_into().unwrap());
            let mut base = 0;
            let irq = val as u32;
            reg.for_each(|x| {
                base = x.starting_address as usize;
            });
            machine.uart[uart_count] = UartInfo {
                base,
                irq: irq as usize,
            };
            uart_count += 1;
        } else if node.name.starts_with(PLIC) {
            let reg = node.reg().unwrap();
            reg.for_each(|x| {
                machine.plic = Range {
                    start: x.starting_address as usize,
                    end: x.starting_address as usize + x.size.unwrap(),
                }
            })
        } else if node.name.starts_with(CLINT) {
            let reg = node.reg().unwrap();
            reg.for_each(|x| {
                machine.clint = Range {
                    start: x.starting_address as usize,
                    end: x.starting_address as usize + x.size.unwrap(),
                }
            })
        } else if node.name.starts_with(RTC) {
            let reg = node.reg().unwrap();
            let irq = 0xc;
            let mut range = 0..0;
            reg.for_each(|x| {
                range = Range {
                    start: x.starting_address as usize,
                    end: x.starting_address as usize + x.size.unwrap(),
                }
            });
            machine.rtc = RtcInfo { range, irq }
        }
    }
    machine.uart_count = uart_count;
    machine
}
