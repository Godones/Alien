pub use context::TaskContext;
use context::TrapFrameRaw;
use corelib::switch_task;
use memory_addr::{PhysAddr, VirtAddr};

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
pub struct TrapFrame(TrapFrameRaw);

impl TrapFrame {
    pub fn new_user(entry: VirtAddr, sp: VirtAddr, k_sp: VirtAddr) -> Self {
        let kernel_satp = corelib::kernel_satp();
        let user_trap_vector = corelib::trap_from_user();
        Self(TrapFrameRaw::init_for_task(
            entry.as_usize(),
            sp.as_usize(),
            kernel_satp,
            k_sp.as_usize(),
            user_trap_vector,
        ))
    }

    pub fn update_k_sp(&mut self, val: VirtAddr) {
        self.0.update_kernel_sp(val.as_usize());
    }

    pub fn update_user_sp(&mut self, val: VirtAddr) {
        self.0.update_user_sp(val.as_usize());
    }

    pub fn update_tp(&mut self, val: VirtAddr) {
        self.0.update_tp(val.as_usize());
    }

    pub fn update_result(&mut self, val: usize) {
        self.0.update_res(val);
    }

    pub fn from_raw_phy_ptr(ptr: PhysAddr) -> &'static mut Self {
        unsafe { &mut *(ptr.as_usize() as *mut usize as *mut Self) }
    }
}
