//！ Trap 上下文 (Trap帧) 的定义和相关操作 
use bit_field::BitField;

use crate::arch::riscv::sstatus::{self, Sstatus, SPP};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TrapFrame {
    /// 整数寄存器组
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
    /// 给出 Trap 发生之前 CPU 处在哪个特权级等信息
    sstatus: Sstatus,
    fg: [usize; 2],
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
            sstatus: Sstatus::default(),
            fg: [0; 2],
        }
    }

    /// 获取当前的 Trap 帧下的 sstatus 寄存器的值
    pub fn get_status(&self) -> Sstatus {
        self.sstatus
    }

    /// 用于在收到外部中断时，需要将程序计数器 pc + 4 (下一条指令位置加4个字节)
    pub fn update_sepc(&mut self) {
        self.sepc += 4;
    }

    /// 
    pub fn from_raw_ptr(ptr: *mut TrapFrame) -> &'static mut Self {
        unsafe { &mut *(ptr as *mut Self) }
    }

    /// 更新 Trap 帧中的内核栈地址
    pub fn update_kernel_sp(&mut self, val: usize) {
        self.k_sp = val;
    }

    /// 返回 Trap 帧中的 sepc 
    pub fn sepc(&self) -> usize {
        self.sepc
    }

    /// 设置 Trap 帧中的 sepc 
    pub fn set_sepc(&mut self, val: usize) {
        self.sepc = val;
    }

    /// 用一个从文件系统中读取到的相关app数据，初始化一个 Trap 帧，使通过其创建的进程在初次进入用户态时能正常运行
    pub fn from_app_info(
        entry: usize,
        sp: usize,
        k_satp: usize,
        k_sp: usize,
        trap_handler: usize,
    ) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spie();
        assert!(sstatus.0.get_bit(5)); //spie == 1
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

    /// 更新 Trap 帧中 x[4] (tp) 的值
    pub fn update_tp(&mut self, val: usize) {
        self.x[4] = val;
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
