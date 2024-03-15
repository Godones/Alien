use core::cmp::min;
use core::fmt::Debug;
use core::ops::Range;

use fdt::Fdt;

const MEMORY: &str = "memory";
const PLIC: &str = "plic";
const CLINT: &str = "clint";
const CHOSE: &str = "chosen";

/// Machine basic information
#[derive(Clone)]
pub struct MachineInfo {
    /// Machine model
    pub model: [u8; 32],
    /// Number of CPUs
    pub smp: usize,
    /// Memory range
    pub memory: Range<usize>,
    /// PLIC information
    pub plic: Range<usize>,
    /// CLINT information
    pub clint: Range<usize>,
    /// Initrd information
    pub initrd: Option<Range<usize>>,
    /// Kernel command line
    pub bootargs: Option<[u8; 255]>,
    pub bootargs_len: usize,
}

impl Debug for MachineInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let index = self.model.iter().position(|&x| x == 0).unwrap_or(32);
        let model = core::str::from_utf8(&self.model[..index]).unwrap();
        write!(
            f,
            "This is a device tree representation of a {} machine\n",
            model
        )
        .unwrap();
        write!(f, "SMP:    {}\n", self.smp).unwrap();
        write!(
            f,
            "Memory: {:#x}..{:#x}\n",
            self.memory.start, self.memory.end
        )
        .unwrap();
        write!(f, "PLIC:   {:#x}..{:#x}\n", self.plic.start, self.plic.end).unwrap();
        write!(
            f,
            "CLINT:  {:#x}..{:#x}\n",
            self.clint.start, self.clint.end
        )
        .unwrap();
        write!(f, "Initrd: {:#x?}\n", self.initrd).unwrap();
        let bootargs = self
            .bootargs
            .as_ref()
            .map(|x| core::str::from_utf8(&x[..self.bootargs_len]).unwrap());
        write!(f, "Bootargs: {:?}", bootargs).unwrap();
        Ok(())
    }
}

/// Get machine information from a device-tree
pub fn machine_info_from_dtb(ptr: usize) -> MachineInfo {
    let fdt = unsafe { Fdt::from_ptr(ptr as *const u8).unwrap() };
    walk_dt(fdt)
}

// Walk the device-tree and get machine information
fn walk_dt(fdt: Fdt) -> MachineInfo {
    let mut machine = MachineInfo {
        model: [0; 32],
        smp: 0,
        memory: 0..0,
        plic: 0..0,
        clint: 0..0,
        initrd: None,
        bootargs: None,
        bootargs_len: 0,
    };
    let x = fdt.root();
    machine.smp = fdt.cpus().count();
    let res = fdt.chosen().bootargs().map(|x| {
        let mut tmp = [0; 255];
        let bootargs = x.as_bytes();
        let len = min(bootargs.len(), tmp.len());
        tmp[0..len].copy_from_slice(&bootargs[..len]);
        (tmp, len)
    });
    if let Some((bootargs, len)) = res {
        machine.bootargs = Some(bootargs);
        machine.bootargs_len = len;
    }
    let model = x.model().as_bytes();
    let len = min(model.len(), machine.model.len());
    machine.model[0..len].copy_from_slice(&model[..len]);
    for node in fdt.all_nodes() {
        if node.name.starts_with(MEMORY) {
            let reg = node.reg().unwrap();
            reg.for_each(|x| {
                machine.memory = Range {
                    start: x.starting_address as usize,
                    end: x.starting_address as usize + x.size.unwrap(),
                }
            })
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
        } else if node.name.starts_with(CHOSE) {
            let initrd_start = node.property("linux,initrd-start");
            if initrd_start.is_none() {
                continue;
            }
            let initrd_start = initrd_start.unwrap();
            let initrd_end = node.property("linux,initrd-end").unwrap();
            let initrd_start = initrd_start.as_usize().unwrap();
            let initrd_end = initrd_end.as_usize().unwrap();
            machine.initrd = Some(Range {
                start: initrd_start,
                end: initrd_end,
            });
        }
    }
    machine
}
