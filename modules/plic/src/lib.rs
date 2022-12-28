#![no_std]

use core::num::NonZeroU32;

const PRIORITY_OFFSET: usize = 0;
#[allow(unused)]
const PENDING_OFFSET: usize = 0x1000;
const ENABLE_OFFSET: usize = 0x2000;
const THRESHOLD_OFFSET: usize = 0x20_0000;
const CLAIM_COMPLETE_OFFSET: usize = 0x20_0004;

#[derive(Debug)]
pub struct PLIC {
    base_addr: usize,
}

// Register	        Address    	Description
// Priority	        0x0c00_0000	Sets the priority of a particular interrupt source
// Pending	        0x0c00_1000	Contains a list of interrupts that have been triggered (are pending)
// Enable	        0x0c00_2000	Enable/disable certain interrupt sources
// Threshold	    0x0c20_0000	Sets the threshold that interrupts must meet before being able to trigger.
// Claim(read)	    0x0c20_0004	Returns the next interrupt in priority order.
// Complete(write) 0x0c20_0004	Completes handling of a particular interrupt.
impl PLIC {
    pub fn new(base_addr: usize) -> Self {
        Self { base_addr }
    }

    pub fn enable(&self, id: u32) {
        let new = 1 << id;
        let addr = (self.base_addr + ENABLE_OFFSET) as *mut u32;
        unsafe {
            let old = addr.read_volatile();
            addr.write_volatile(old | new)
        }
    }

    pub fn set_priority(&self, id: u32, priority: u8) {
        let pri = priority as u32 & 7;
        let addr = (self.base_addr + PRIORITY_OFFSET) as *mut u32;
        unsafe {
            addr.add(id as usize).write_volatile(pri);
        }
    }

    pub fn set_threshold(&self, threshold: u32) {
        let threshold = threshold & 7;
        let addr = (self.base_addr + THRESHOLD_OFFSET) as *mut u32;
        unsafe {
            addr.write_volatile(threshold);
        }
    }

    pub fn claim(&self) -> Option<NonZeroU32> {
        let addr = (self.base_addr + CLAIM_COMPLETE_OFFSET) as *mut u32;
        let val = unsafe { addr.read_volatile() };
        return if val == 0 { None } else { NonZeroU32::new(val) };
    }

    pub fn complete(&self, id: u32) {
        let addr = (self.base_addr + CLAIM_COMPLETE_OFFSET) as *mut u32;
        unsafe {
            addr.write_volatile(id);
        }
    }
}
