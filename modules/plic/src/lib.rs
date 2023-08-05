#![cfg_attr(not(test), no_std)]

use core::num::NonZeroU32;

const PRIORITY_OFFSET: usize = 0;
const PENDING_OFFSET: usize = 0x1000;
const ENABLE_OFFSET: usize = 0x2000;
const THRESHOLD_OFFSET: usize = 0x20_0000;
const CLAIM_COMPLETE_OFFSET: usize = 0x20_0004;

const HART_NUM: usize = 32;

#[derive(Debug)]
pub struct PLIC {
    base_addr: usize,
    privileges: [u8; HART_NUM],
}

// Register	        Address    	Description
// Priority	        0x0c00_0000	Sets the priority of a particular interrupt source
// Pending	        0x0c00_1000	Contains a list of interrupts that have been triggered (are pending)
// Enable	        0x0c00_2000	Enable/disable certain interrupt sources
// Threshold	    0x0c20_0000	Sets the threshold that interrupts must meet before being able to trigger.
// Claim(read)	    0x0c20_0004	Returns the next interrupt in priority order.
// Complete(write) 0x0c20_0004	Completes handling of a particular interrupt.

#[derive(Debug)]
pub enum Mode {
    Machine = 0,
    Supervisor = 1,
}

impl PLIC {
    pub fn new(base_addr: usize, privileges: &[u8]) -> Self {
        assert!(privileges.len() < HART_NUM);
        let mut tmp = [0u8; 32];
        tmp[..privileges.len()].copy_from_slice(privileges);
        Self {
            base_addr,
            privileges: tmp,
        }
    }

    pub fn enable(&self, hart: u32, mode: Mode, irq: u32) {
        let contexts = self.check(hart, mode, irq);
        let index = (irq / 32) as usize;
        let bit = (irq % 32) as usize;
        let addr = (self.base_addr + ENABLE_OFFSET + contexts * 0x80) + index * 4;
        let addr = addr as *mut u32;
        write(addr, read(addr) | (1 << bit));
    }

    pub fn disable(&self, hart: u32, mode: Mode, irq: u32) {
        let contexts = self.check(hart, mode, irq);
        let index = (irq / 32) as usize;
        let bit = (irq % 32) as usize;
        let addr = (self.base_addr + ENABLE_OFFSET + contexts * 0x80) + index * 4;
        let addr = addr as *mut u32;
        write(addr, read(addr) & !(1 << bit));
    }
    /// check if the interrupt is pending
    pub fn pending(&self, irq: u32) -> bool {
        assert!(irq < 1024);
        let addr = (self.base_addr + PENDING_OFFSET) as *mut u32;
        let index = (irq / 32) as usize;
        let bit = (irq % 32) as usize;
        let val = unsafe { addr.add(index as usize).read_volatile() };
        return (val & (1 << bit)) != 0;
    }

    pub fn set_priority(&self, irq: u32, priority: u32) {
        assert!(irq < 1024);
        let pri = priority & 7;
        let addr = (self.base_addr + PRIORITY_OFFSET) as *mut u32;
        unsafe {
            addr.add(irq as usize).write_volatile(pri);
        }
    }

    pub fn set_threshold(&self, hart: u32, mode: Mode, threshold: u32) {
        let contexts = self.check(hart, mode, 0);
        let addr = (self.base_addr + THRESHOLD_OFFSET + contexts * 0x1000) as *mut u32;
        let threshold = threshold & 7;
        write(addr, threshold);
    }

    pub fn claim(&self, hart: u32, mode: Mode) -> Option<NonZeroU32> {
        let contexts = self.check(hart, mode, 0);
        let addr = (self.base_addr + CLAIM_COMPLETE_OFFSET + contexts * 0x1000) as *mut u32;
        Some(NonZeroU32::new(read(addr)).unwrap())
    }

    pub fn complete(&self, hart: u32, mode: Mode, irq: u32) {
        let contexts = self.check(hart, mode, irq);
        let addr = (self.base_addr + CLAIM_COMPLETE_OFFSET + contexts * 0x1000) as *mut u32;
        write(addr, irq);
    }

    fn check(&self, hart: u32, mode: Mode, irq: u32) -> usize {
        assert!(hart < HART_NUM as u32);
        let privileges = self.privileges[hart as usize];
        let mode = mode as u8;
        assert!(mode <= privileges);
        assert!(irq < 1024);
        let contexts = self.privileges[..hart as usize]
            .iter()
            .map(|x| *x as usize)
            .fold(0, |acc, x| acc + x);
        contexts + mode as usize
    }
}

fn write(addr: *mut u32, val: u32) {
    unsafe {
        addr.write_volatile(val);
    }
}

fn read(addr: *const u32) -> u32 {
    unsafe { addr.read_volatile() }
}
