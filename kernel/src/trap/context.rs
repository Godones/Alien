#[repr(C)]
#[derive(Debug)]
pub struct TrapFrame {
    pub x: [usize; 32],
    // sepc 记录陷入地址
    pub sepc: usize,
    // k_satp 记录内核根页表地址
    pub k_satp: usize,
    // k_sp记录task内核栈地址
    pub k_sp: usize,
    // 记录trap处理的地址
    pub trap_handler: usize,
    // 记录所在的核
    pub hart_id: usize,
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
}
