#[repr(C)]
#[derive(Debug)]
pub struct TrapFrame {
    x: [usize; 32],
    /// sepc 记录陷入地址
    sepc: usize,
    /// k_satp 记录内核根页表地址
    k_satp: usize,
    /// k_sp记录task内核栈地址
    k_sp: usize,
    /// 记录trap处理的地址
    trap_handler: usize,
    /// 记录所在的核
    hart_id: usize,
}

impl TrapFrame {
    pub fn empty() -> Self {
        Self {
            x: [0; 32],
            sepc: 0,
            k_satp: 0,
            k_sp: 0,
            trap_handler: 0,
            hart_id: 0,
        }
    }
    pub fn update_sepc(&mut self) {
        self.sepc += 4;
    }

    pub fn from_raw_ptr(ptr: *mut TrapFrame) -> &'static mut Self {
        unsafe { &mut *(ptr as *mut Self) }
    }
    pub fn update_kernel_sp(&mut self, val: usize) {
        self.k_sp = val;
    }

    pub fn from_app_info(
        entry: usize,
        sp: usize,
        k_satp: usize,
        k_sp: usize,
        trap_handler: usize,
    ) -> Self {
        let mut res = Self {
            x: [0; 32],
            sepc: entry,
            k_satp,
            k_sp,
            trap_handler,
            hart_id: 0,
        };
        res.x[2] = sp;
        res
    }
    pub fn update_res(&mut self, val: usize) {
        self.x[10] = val;
    }
    pub fn parameters(&self) -> [usize; 4] {
        [self.x[17], self.x[10], self.x[11], self.x[12]]
    }
    pub fn regs(&mut self) -> &mut [usize] {
        &mut self.x
    }
}
