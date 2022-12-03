#[repr(C)]
#[derive(Debug)]
pub struct TrapContext {
    pub x: [usize; 32],
    // sepc 记录陷入地址
    pub sepc: usize,
    // k_satp 记录内核根页表地址
    pub k_satp: usize,
    // k_sp记录app内核栈地址
    pub k_sp: usize,
    // 记录处理地址
    pub trap_handler: usize,
    // 记录所在的核
    pub hart_id: usize,
}
