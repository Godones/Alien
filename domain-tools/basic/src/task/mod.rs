use arch::ExtSstatus;
use corelib::switch_task;
use memory_addr::{PhysAddr, VirtAddr};
use riscv::register::sstatus::SPP;
pub use task_meta::TaskContext;

pub fn switch(now: *mut TaskContext, next: *const TaskContext, next_tid: usize) {
    switch_task(now, next, next_tid)
}

pub trait TaskContextExt {
    fn new_user(k_sp: VirtAddr) -> Self;
    fn new_kernel(func_ptr: *const (), k_sp: VirtAddr) -> Self;
}

impl TaskContextExt for TaskContext {
    fn new_user(k_sp: VirtAddr) -> Self {
        TaskContext::new(corelib::trap_to_user(), k_sp.as_usize())
    }
    fn new_kernel(func_ptr: *const (), k_sp: VirtAddr) -> Self {
        TaskContext::new(func_ptr as usize, k_sp.as_usize())
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TrapFrame {
    x: [usize; 32],
    sepc: usize,
    k_satp: usize,
    k_sp: usize,
    trap_handler: usize,
    hart_id: usize,
    sstatus: ExtSstatus,
    fg: [usize; 2],
}

impl TrapFrame {
    fn init_for_task(
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
}

impl TrapFrame {
    pub fn new_user(entry: VirtAddr, sp: VirtAddr, k_sp: VirtAddr) -> Self {
        let kernel_satp = corelib::kernel_satp();
        let user_trap_vector = corelib::trap_from_user();
        Self::init_for_task(
            entry.as_usize(),
            sp.as_usize(),
            kernel_satp,
            k_sp.as_usize(),
            user_trap_vector,
        )
    }
    pub fn update_k_sp(&mut self, val: VirtAddr) {
        self.k_sp = val.as_usize();
    }

    pub fn update_user_sp(&mut self, val: VirtAddr) {
        self.x[2] = val.as_usize();
    }

    pub fn update_tp(&mut self, val: VirtAddr) {
        self.x[4] = val.as_usize();
    }

    pub fn update_result(&mut self, val: usize) {
        self.x[10] = val;
    }

    pub fn sepc(&self) -> VirtAddr {
        VirtAddr::from(self.sepc)
    }
    pub fn update_sepc(&mut self, val: VirtAddr) {
        self.sepc = val.as_usize();
    }
    pub fn from_raw_phy_ptr(ptr: PhysAddr) -> &'static mut Self {
        unsafe { &mut *(ptr.as_usize() as *mut usize as *mut Self) }
    }
    pub fn parameters(&self) -> [usize; 7] {
        [
            self.x[17], self.x[10], self.x[11], self.x[12], self.x[13], self.x[14], self.x[15],
        ]
    }
}
