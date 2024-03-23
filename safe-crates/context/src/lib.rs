#![no_std]
/// 线程切换需要保存的上下文
///
/// 线程切换由__switch()完成，这个汇编函数不会由编译器完成寄存器保存，因此需要手动保存
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct TaskContext {
    /// ra 寄存器
    ra: usize,
    /// sp 寄存器值
    sp: usize,
    /// s0 ~ s11
    s: [usize; 12],
}

impl TaskContext {
    /// 创建一个新的上下文，默认 s0 ~ s11 的值为 0
    pub fn new(ra: usize, sp: usize) -> Self {
        Self { ra, sp, s: [0; 12] }
    }

    /// 创建一个全为 0 的上下文
    pub const fn empty() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }
}

use arch::ExtSstatus;
use riscv::register::sstatus::SPP;

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
    /// 获取当前的 Trap 帧下的 sstatus 寄存器的值
    pub fn get_status(&self) -> ExtSstatus {
        self.sstatus
    }

    /// 用于在收到外部中断时，需要将程序计数器 pc + 4 (下一条指令位置加4个字节)
    pub fn update_sepc(&mut self) {
        self.sepc += 4;
    }

    pub fn from_raw_ptr(ptr: *mut TrapFrame) -> &'static mut Self {
        unsafe { &mut *(ptr) }
    }

    /// 更新 Trap 帧中的内核栈地址
    pub fn update_kernel_sp(&mut self, val: usize) {
        self.k_sp = val;
    }

    pub fn update_user_sp(&mut self, val: usize) {
        self.x[2] = val;
    }

    /// 返回 Trap 帧中的 sepc
    pub fn sepc(&self) -> usize {
        self.sepc
    }

    /// 更新 Trap 帧中 x[4] (tp) 的值
    pub fn update_tp(&mut self, val: usize) {
        self.x[4] = val;
    }

    /// 设置 Trap 帧中的 sepc
    pub fn set_sepc(&mut self, val: usize) {
        self.sepc = val;
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

    /// 更新 Trap 帧中 x[10] (即函数返回值) 的值
    pub fn update_res(&mut self, val: usize) {
        self.x[10] = val;
    }

    /// 获取系统调用的参数，一般用于发生 trap 的原因是系统调用时
    pub fn parameters(&self) -> [usize; 7] {
        [
            self.x[17], self.x[10], self.x[11], self.x[12], self.x[13], self.x[14], self.x[15],
        ]
    }
    /// 获取整数寄存器组的可变引用
    pub fn regs(&mut self) -> &mut [usize] {
        &mut self.x
    }
}
