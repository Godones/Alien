//! # PLIC
//! This crate provides a platform-level interrupt controller (PLIC) driver for RISC-V.
#![cfg_attr(not(test), no_std)]
#![deny(missing_docs)]

// Qemu PLIC mapping
// Register	        Address    	Description
// Priority	        0x0c00_0000	Sets the priority of a particular interrupt source
// Pending	        0x0c00_1000	Contains a list of interrupts that have been triggered (are pending)
// Enable	        0x0c00_2000	Enable/disable certain interrupt sources
// Threshold	    0x0c20_0000	Sets the threshold that interrupts must meet before being able to trigger.
// Claim(read)	    0x0c20_0004	Returns the next interrupt in priority order.
// Complete(write) 0x0c20_0004	Completes handling of a particular interrupt.

const PRIORITY_OFFSET: usize = 0;
const PENDING_OFFSET: usize = 0x1000;
const ENABLE_OFFSET: usize = 0x2000;
const THRESHOLD_OFFSET: usize = 0x20_0000;
const CLAIM_COMPLETE_OFFSET: usize = 0x20_0004;

/// The maximum number of contexts that can be supported by the PLIC.
const MAX_CONTEXT: usize = 15872;
/// The maximum number of interrupt sources that can be supported by the PLIC.
const MAX_INTERRUPT: usize = 1024;

/// The PLIC is a platform-level interrupt controller. It connects all external interrupts in the
/// system to all hart contexts in the system. The PLIC is designed to support multiple harts, each
/// with its own context, connected to a set of platform interrupt sources. The PLIC supports
/// 1,024 external interrupt sources and up to 15872 contexts. Each external interrupt source can
/// be individually masked and has a programmable priority level.
#[derive(Debug)]
pub struct PLIC<const H: usize> {
    base_addr: usize,
    privileges: [u8; H],
}

/// The interrupt mode.
#[derive(Debug)]
pub enum Mode {
    /// Machine mode
    Machine = 0,
    /// Supervisor mode
    Supervisor = 1,
}

impl<const H: usize> PLIC<H> {
    /// Create a new PLIC instance.
    pub fn new(base_addr: usize, privileges: [u8; H]) -> Self {
        Self {
            base_addr,
            privileges,
        }
    }

    /// enable the interrupt
    ///
    /// # Parameters
    /// hart: the hart id
    ///
    /// mode: the interrupt mode
    ///
    /// irq: the interrupt id
    pub fn enable(&self, hart: u32, mode: Mode, irq: u32) {
        assert!(irq < MAX_INTERRUPT as u32);
        let contexts = self.context_index(hart, mode);
        let index = (irq / 32) as usize;
        let bit = (irq % 32) as usize;
        let addr = (self.base_addr + ENABLE_OFFSET + contexts * 0x80) + index * 4;
        let addr = addr as *mut u32;
        write(addr, read(addr) | (1 << bit));
    }

    /// disable the interrupt
    ///
    /// The parameter is the same as the enable function
    pub fn disable(&self, hart: u32, mode: Mode, irq: u32) {
        assert!(irq < MAX_INTERRUPT as u32);
        let contexts = self.context_index(hart, mode);
        let index = (irq / 32) as usize;
        let bit = (irq % 32) as usize;
        let addr = (self.base_addr + ENABLE_OFFSET + contexts * 0x80) + index * 4;
        let addr = addr as *mut u32;
        write(addr, read(addr) & !(1 << bit));
    }
    /// check if the interrupt is pending
    pub fn pending(&self, irq: u32) -> bool {
        assert!(irq < MAX_INTERRUPT as u32);
        let index = (irq / 32) as usize;
        let bit = (irq % 32) as usize;
        let addr = self.base_addr + PENDING_OFFSET + index * 4;
        let val = read(addr as *const u32);
        return (val & (1 << bit)) != 0;
    }

    /// set the priority of the interrupt
    ///
    /// # Parameters
    /// irq: the interrupt id
    /// priority: the priority of the interrupt, the value should be in \[0,7]
    pub fn set_priority(&self, irq: u32, priority: u32) {
        assert!(irq < MAX_INTERRUPT as u32);
        assert!(priority < 8);
        let pri = priority & 7;
        let addr = self.base_addr + PRIORITY_OFFSET + irq as usize * 4;
        write(addr as *mut u32, pri);
    }

    /// set the threshold for the hart context
    pub fn set_threshold(&self, hart: u32, mode: Mode, threshold: u32) {
        let contexts = self.context_index(hart, mode);
        let addr = (self.base_addr + THRESHOLD_OFFSET + contexts * 0x1000) as *mut u32;
        write(addr, threshold);
    }

    /// get the next pending interrupt
    pub fn claim(&self, hart: u32, mode: Mode) -> u32 {
        let contexts = self.context_index(hart, mode);
        let addr = (self.base_addr + CLAIM_COMPLETE_OFFSET + contexts * 0x1000) as *mut u32;
        read(addr)
    }

    /// complete the interrupt
    pub fn complete(&self, hart: u32, mode: Mode, irq: u32) {
        assert!(irq < MAX_INTERRUPT as u32);
        let contexts = self.context_index(hart, mode);
        let addr = (self.base_addr + CLAIM_COMPLETE_OFFSET + contexts * 0x1000) as *mut u32;
        write(addr, irq);
    }

    /// calculate the context index
    fn context_index(&self, hart: u32, mode: Mode) -> usize {
        assert!(hart < H as u32);
        let privileges = self.privileges[hart as usize];
        let mode = mode as u8;
        assert!(mode <= privileges);
        let contexts = self.privileges[..hart as usize]
            .iter()
            .map(|x| *x as usize)
            .fold(0, |acc, x| acc + x);
        let res = contexts + mode as usize;
        assert!(res < MAX_CONTEXT);
        res
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
