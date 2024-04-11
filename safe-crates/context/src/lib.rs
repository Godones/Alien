#![no_std]
use arch::ExtSstatus;
use riscv::register::sstatus::SPP;
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct TaskContext {
    /// ra
    ra: usize,
    /// sp
    sp: usize,
    /// s0 ~ s11
    s: [usize; 12],
}

impl TaskContext {
    pub fn new(ra: usize, sp: usize) -> Self {
        Self { ra, sp, s: [0; 12] }
    }

    pub const fn empty() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TrapFrameRaw {
    x: [usize; 32],
    sepc: usize,
    k_satp: usize,
    k_sp: usize,
    trap_handler: usize,
    hart_id: usize,
    sstatus: ExtSstatus,
    fg: [usize; 2],
}

impl TrapFrameRaw {
    pub fn sepc(&self) -> usize {
        self.sepc
    }
    pub fn update_sepc(&mut self, val: usize) {
        self.sepc = val;
    }

    pub fn from_raw_ptr(ptr: *mut TrapFrameRaw) -> &'static mut Self {
        unsafe { &mut *(ptr) }
    }

    pub fn update_kernel_sp(&mut self, val: usize) {
        self.k_sp = val;
    }

    pub fn update_user_sp(&mut self, val: usize) {
        self.x[2] = val;
    }

    pub fn update_tp(&mut self, val: usize) {
        self.x[4] = val;
    }

    pub fn init_for_task(
        entry: usize,
        sp: usize,
        k_satp: usize,
        k_sp: usize,
        trap_handler: usize,
    ) -> Self {
        let mut sstatus = ExtSstatus::read();
        sstatus.set_spie();
        sstatus.set_spp(SPP::User);
        sstatus.set_sie(false);
        let mut res = Self {
            x: [0; 32],
            sepc: entry,
            k_satp,
            k_sp,
            trap_handler,
            hart_id: 0,
            sstatus,
            fg: [0; 2],
        };
        res.x[2] = sp;
        res
    }

    pub fn update_res(&mut self, val: usize) {
        self.x[10] = val;
    }

    pub fn parameters(&self) -> [usize; 7] {
        [
            self.x[17], self.x[10], self.x[11], self.x[12], self.x[13], self.x[14], self.x[15],
        ]
    }
}
