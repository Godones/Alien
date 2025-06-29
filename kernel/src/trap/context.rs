//! Trap 上下文 (Trap帧) 的定义和相关操作

use arch::ExtSstatus;
use kprobe::PtRegs;
use riscv::register::sstatus::SPP;

#[repr(C)]
#[derive(Debug, Clone)]
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
    sstatus: ExtSstatus,
    fg: [usize; 2],
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct KTrapFrame {
    x: [usize; 32],
    sstatus: ExtSstatus,
    sepc: usize,
}

#[derive(Debug)]
pub enum CommonTrapFrame {
    Kernel(&'static mut KTrapFrame),
    User(&'static mut TrapFrame),
}

impl CommonTrapFrame {
    pub fn update_from_pt_regs(&mut self, pt_regs: &PtRegs) {
        match self {
            CommonTrapFrame::Kernel(ktrap) => ktrap.update_from_pt_regs(pt_regs),
            CommonTrapFrame::User(utrap) => utrap.update_from_pt_regs(pt_regs),
        }
    }

    pub fn to_pt_regs(&self) -> PtRegs {
        match self {
            CommonTrapFrame::Kernel(frame) => PtRegs {
                epc: frame.sepc,
                ra: frame.x[1],
                sp: frame.x[2],
                gp: frame.x[3],
                tp: frame.x[4],
                t0: frame.x[5],
                t1: frame.x[6],
                t2: frame.x[7],
                s0: frame.x[8],
                s1: frame.x[9],
                a0: frame.x[10],
                a1: frame.x[11],
                a2: frame.x[12],
                a3: frame.x[13],
                a4: frame.x[14],
                a5: frame.x[15],
                a6: frame.x[16],
                a7: frame.x[17],
                s2: frame.x[18],
                s3: frame.x[19],
                s4: frame.x[20],
                s5: frame.x[21],
                s6: frame.x[22],
                s7: frame.x[23],
                s8: frame.x[24],
                s9: frame.x[25],
                s10: frame.x[26],
                s11: frame.x[27],
                t3: frame.x[28],
                t4: frame.x[29],
                t5: frame.x[30],
                t6: frame.x[31],
                status: frame.sstatus.0,
                badaddr: 0,
                cause: 0,
                orig_a0: 0,
            },
            CommonTrapFrame::User(frame) => PtRegs {
                epc: frame.sepc,
                ra: frame.x[1],
                sp: frame.x[2],
                gp: frame.x[3],
                tp: frame.x[4],
                t0: frame.x[5],
                t1: frame.x[6],
                t2: frame.x[7],
                s0: frame.x[8],
                s1: frame.x[9],
                a0: frame.x[10],
                a1: frame.x[11],
                a2: frame.x[12],
                a3: frame.x[13],
                a4: frame.x[14],
                a5: frame.x[15],
                a6: frame.x[16],
                a7: frame.x[17],
                s2: frame.x[18],
                s3: frame.x[19],
                s4: frame.x[20],
                s5: frame.x[21],
                s6: frame.x[22],
                s7: frame.x[23],
                s8: frame.x[24],
                s9: frame.x[25],
                s10: frame.x[26],
                s11: frame.x[27],
                t3: frame.x[28],
                t4: frame.x[29],
                t5: frame.x[30],
                t6: frame.x[31],
                status: frame.sstatus.0,
                badaddr: 0,
                cause: 0,
                orig_a0: 0,
            },
        }
    }
}

impl KTrapFrame {
    /// 返回 Trap 帧中的 sepc
    pub fn sepc(&self) -> usize {
        self.sepc
    }

    /// 设置 Trap 帧中的 sepc
    pub fn set_sepc(&mut self, val: usize) {
        self.sepc = val;
    }

    pub fn update_from_pt_regs(&mut self, pt_regs: &PtRegs) {
        let frame = self;
        frame.sepc = pt_regs.epc;
        frame.x[1] = pt_regs.ra;
        frame.x[2] = pt_regs.sp;
        frame.x[3] = pt_regs.gp;
        frame.x[4] = pt_regs.tp;
        frame.x[5] = pt_regs.t0;
        frame.x[6] = pt_regs.t1;
        frame.x[7] = pt_regs.t2;
        frame.x[8] = pt_regs.s0;
        frame.x[9] = pt_regs.s1;
        frame.x[10] = pt_regs.a0;
        frame.x[11] = pt_regs.a1;
        frame.x[12] = pt_regs.a2;
        frame.x[13] = pt_regs.a3;
        frame.x[14] = pt_regs.a4;
        frame.x[15] = pt_regs.a5;
        frame.x[16] = pt_regs.a6;
        frame.x[17] = pt_regs.a7;
        frame.x[18] = pt_regs.s2;
        frame.x[19] = pt_regs.s3;
        frame.x[20] = pt_regs.s4;
        frame.x[21] = pt_regs.s5;
        frame.x[22] = pt_regs.s6;
        frame.x[23] = pt_regs.s7;
        frame.x[24] = pt_regs.s8;
        frame.x[25] = pt_regs.s9;
        frame.x[26] = pt_regs.s10;
        frame.x[27] = pt_regs.s11;
        frame.x[28] = pt_regs.t3;
        frame.x[29] = pt_regs.t4;
        frame.x[30] = pt_regs.t5;
        frame.x[31] = pt_regs.t6;
        frame.sstatus.set_value(pt_regs.status);
    }
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

    /// 返回 Trap 帧中的 sepc
    pub fn sepc(&self) -> usize {
        self.sepc
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
        // assert!(sstatus.0.get_bit(5)); // spie == 1
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

    pub fn update_from_pt_regs(&mut self, pt_regs: &PtRegs) {
        let frame = self;
        frame.sepc = pt_regs.epc;
        frame.x[1] = pt_regs.ra;
        frame.x[2] = pt_regs.sp;
        frame.x[3] = pt_regs.gp;
        frame.x[4] = pt_regs.tp;
        frame.x[5] = pt_regs.t0;
        frame.x[6] = pt_regs.t1;
        frame.x[7] = pt_regs.t2;
        frame.x[8] = pt_regs.s0;
        frame.x[9] = pt_regs.s1;
        frame.x[10] = pt_regs.a0;
        frame.x[11] = pt_regs.a1;
        frame.x[12] = pt_regs.a2;
        frame.x[13] = pt_regs.a3;
        frame.x[14] = pt_regs.a4;
        frame.x[15] = pt_regs.a5;
        frame.x[16] = pt_regs.a6;
        frame.x[17] = pt_regs.a7;
        frame.x[18] = pt_regs.s2;
        frame.x[19] = pt_regs.s3;
        frame.x[20] = pt_regs.s4;
        frame.x[21] = pt_regs.s5;
        frame.x[22] = pt_regs.s6;
        frame.x[23] = pt_regs.s7;
        frame.x[24] = pt_regs.s8;
        frame.x[25] = pt_regs.s9;
        frame.x[26] = pt_regs.s10;
        frame.x[27] = pt_regs.s11;
        frame.x[28] = pt_regs.t3;
        frame.x[29] = pt_regs.t4;
        frame.x[30] = pt_regs.t5;
        frame.x[31] = pt_regs.t6;
        frame.sstatus.set_value(pt_regs.status);
    }
}
