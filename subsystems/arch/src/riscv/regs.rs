use core::arch::asm;

use riscv::register::sstatus::SPP;

#[derive(Debug, Default, Copy, Clone)]
pub struct ExtSstatus(pub usize);

impl ExtSstatus {
    pub fn read() -> Self {
        let val: usize;
        unsafe {
            asm!("csrr {},sstatus", out(reg)val);
        }
        ExtSstatus(val)
    }
    pub fn set_value(&mut self, val: usize) {
        self.0 = val;
    }
    pub fn set_spp(&mut self, spp: SPP) {
        // self.0.set_bit(8, spp as usize != 0);
        self.0 = self.0 & !(1 << 8) | ((spp as usize) << 8);
    }
    pub fn spp(&self) -> SPP {
        let v = (self.0 >> 8) & 1;
        if v == 1 {
            SPP::Supervisor
        } else {
            SPP::User
        }
    }
    pub fn set_spie(&mut self) {
        self.0 |= 1 << 5;
    }
    pub fn sie(&self) -> bool {
        (self.0 & (1 << 1)) != 0
    }
    pub fn set_sie(&mut self, value: bool) {
        self.0 = self.0 & !(1 << 1) | ((value as usize) << 1);
    }
}
