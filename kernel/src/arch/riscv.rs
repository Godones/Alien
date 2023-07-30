pub use riscv::*;

pub mod sstatus {
    use core::arch::asm;

    use bit_field::BitField;
    pub use riscv::register::sstatus::*;

    #[derive(Copy, Clone, Debug, Default)]
    pub struct Sstatus(pub usize);

    impl Sstatus {
        pub fn set_value(&mut self, val: usize) {
            self.0 = val;
        }
        pub fn set_spp(&mut self, spp: SPP) {
            self.0.set_bit(8, spp as usize != 0);
        }
        pub fn spp(&self) -> SPP {
            let v = self.0.get_bit(8);
            if v {
                SPP::Supervisor
            } else {
                SPP::User
            }
        }
        pub fn set_spie(&mut self) {
            self.0.set_bit(5, true);
        }
        pub fn sie(&self) -> bool {
            self.0.get_bit(1)
        }
        pub fn set_sie(&mut self, value: bool) {
            self.0.set_bit(1, value);
        }
    }

    pub fn read() -> Sstatus {
        let val: usize;
        unsafe {
            asm!("csrr {},sstatus", out(reg)val);
        }
        Sstatus(val)
    }
}
