#[repr(C)]
#[derive(Clone, Copy)]
pub struct TrapFrame {
    pub regs: [usize; 32],   // 0 - 255
    pub f_regs: [usize; 32], // 256 - 511
    pub satp: usize,         // 512 - 519
    pub trap_stack: *mut u8, // 520
    pub hart_id: usize,      // 528
}
