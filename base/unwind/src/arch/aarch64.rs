use gimli::Register;

use crate::{arch::registers::Registers, registers};

/// The calling convention dictates the following order of arguments:
/// * first arg in `x0` register, the pointer of the UnwindingContext,
/// * second arg in `x1` register, the stack pointer,
/// * third arg in `x2` register, the saved register values used to recover execution context
///   after we change the register values during unwinding,
#[no_mangle]
#[allow(improper_ctypes_definitions)]
unsafe extern "C" fn unwind_recorder(
    ctx: *mut super::UnwindingContext,
    stack: u64,
    saved_regs: *mut CalleeSavedRegs,
) {
    let saved_regs = &*saved_regs;
    let mut registers = Registers::default();

    registers[Aarch64::X19] = Some(saved_regs.r[0]);
    registers[Aarch64::X20] = Some(saved_regs.r[1]);
    registers[Aarch64::X21] = Some(saved_regs.r[2]);
    registers[Aarch64::X22] = Some(saved_regs.r[3]);
    registers[Aarch64::X23] = Some(saved_regs.r[4]);
    registers[Aarch64::X24] = Some(saved_regs.r[5]);
    registers[Aarch64::X25] = Some(saved_regs.r[6]);
    registers[Aarch64::X26] = Some(saved_regs.r[7]);
    registers[Aarch64::X27] = Some(saved_regs.r[8]);
    registers[Aarch64::X28] = Some(saved_regs.r[9]);
    registers[Aarch64::X29] = Some(saved_regs.r[10]);
    registers[Aarch64::SP] = Some(stack);
    registers[Aarch64::X30] = Some(saved_regs.lr);

    super::unwind_from_panic_stub(registers, ctx);
}

#[naked]
pub unsafe extern "C" fn unwind_trampoline(ctx: usize) {
    core::arch::asm!(
        "mov x1, sp",
        "sub sp, sp, 0xA0",
        ".cfi_adjust_cfa_offset 0x60",
        "stp x19, x20, [sp, #0x00]",
        "stp x21, x22, [sp, #0x10]",
        "stp x23, x24, [sp, #0x20]",
        "stp x25, x26, [sp, #0x30]",
        "stp x27, x28, [sp, #0x40]",
        "stp x29, lr,  [sp, #0x50]",
        ".cfi_rel_offset lr, 0x58",
        "mov x2, sp",
        "bl unwind_recorder",
        "ldr lr, [sp, #0x58]",
        ".cfi_restore lr",
        "add sp, sp, 0x60",
        ".cfi_adjust_cfa_offset -0x60",
        "ret",
        options(noreturn),
    )
}

#[naked]
unsafe extern "C" fn unwind_lander(regs: *const LandingRegs) -> ! {
    core::arch::asm!(
        "ldp x2,  x3,  [x0, #0x10]",
        "ldp x4,  x5,  [x0, #0x20]",
        "ldp x6,  x7,  [x0, #0x30]",
        "ldp x8,  x9,  [x0, #0x40]",
        "ldp x10, x11, [x0, #0x50]",
        "ldp x12, x13, [x0, #0x60]",
        "ldp x14, x15, [x0, #0x70]",
        "ldp x16, x17, [x0, #0x80]",
        "ldp x18, x19, [x0, #0x90]",
        "ldp x20, x21, [x0, #0xA0]",
        "ldp x22, x23, [x0, #0xB0]",
        "ldp x24, x25, [x0, #0xC0]",
        "ldp x26, x27, [x0, #0xD0]",
        "ldp x28, x29, [x0, #0xE0]",
        "ldp x30, x1,  [x0, #0xF0]",
        "mov sp, x1",
        "ldp x0,  x1,  [x0, #0x00]",
        "ret",
        options(noreturn),
    )
}

#[naked]
unsafe extern "C" fn unwind_lander_from_exception(regs: *const LandingRegs) -> ! {
    core::arch::asm!(
        "ldp x2,  x3,  [x0, #0x10]",
        "ldp x4,  x5,  [x0, #0x20]",
        "ldp x6,  x7,  [x0, #0x30]",
        "ldp x8,  x9,  [x0, #0x40]",
        "ldp x10, x11, [x0, #0x50]",
        "ldp x12, x13, [x0, #0x60]",
        "ldp x14, x15, [x0, #0x70]",
        "ldp x16, x17, [x0, #0x80]",
        "ldp x18, x19, [x0, #0x90]",
        "ldp x20, x21, [x0, #0xA0]",
        "ldp x22, x23, [x0, #0xB0]",
        "ldp x24, x25, [x0, #0xC0]",
        "ldp x26, x27, [x0, #0xD0]",
        "ldp x28, x29, [x0, #0xE0]",
        "ldp x30, x1,  [x0, #0xF0]",
        "msr elr_el1, x30",
        "msr sp_el0, x1",
        "mov x1, #0x45",
        "msr spsr_el1, x1",
        "ldp x0,  x1,  [x0, #0x00]",
        "eret",
        options(noreturn),
    )
}

/// **Landing** refers to the process of jumping to a handler for a stack frame,
/// e.g., an unwinding cleanup function, or an exception "catch" block.
///
/// This function basically fills the actual CPU registers with the values in the given `LandingRegisters`
/// and then jumps to the exception handler (landing pad) pointed to by the stack pointer (sp) in those `LandingRegisters`.
///
/// This is similar in design to how the latter half of a context switch routine
/// must restore the previously-saved registers for the next task.
pub unsafe fn land(regs: &Registers, landing_pad_address: u64, from_exception: bool) {
    let mut lr = LandingRegs {
        x: [0; 29],
        fp: regs[Aarch64::X29].unwrap_or(0),
        lr: landing_pad_address,
        sp: regs[Aarch64::SP].unwrap_or(0),
    };
    lr.x[0] = regs[Aarch64::X0].unwrap_or(0);
    lr.x[1] = regs[Aarch64::X1].unwrap_or(0);
    lr.x[2] = regs[Aarch64::X2].unwrap_or(0);
    lr.x[3] = regs[Aarch64::X3].unwrap_or(0);
    lr.x[4] = regs[Aarch64::X4].unwrap_or(0);
    lr.x[5] = regs[Aarch64::X5].unwrap_or(0);
    lr.x[6] = regs[Aarch64::X6].unwrap_or(0);
    lr.x[7] = regs[Aarch64::X7].unwrap_or(0);
    lr.x[8] = regs[Aarch64::X8].unwrap_or(0);
    lr.x[9] = regs[Aarch64::X9].unwrap_or(0);
    lr.x[10] = regs[Aarch64::X10].unwrap_or(0);
    lr.x[11] = regs[Aarch64::X11].unwrap_or(0);
    lr.x[12] = regs[Aarch64::X12].unwrap_or(0);
    lr.x[13] = regs[Aarch64::X13].unwrap_or(0);
    lr.x[14] = regs[Aarch64::X14].unwrap_or(0);
    lr.x[15] = regs[Aarch64::X15].unwrap_or(0);
    lr.x[16] = regs[Aarch64::X16].unwrap_or(0);
    lr.x[17] = regs[Aarch64::X17].unwrap_or(0);
    lr.x[18] = regs[Aarch64::X18].unwrap_or(0);
    lr.x[19] = regs[Aarch64::X19].unwrap_or(0);
    lr.x[20] = regs[Aarch64::X20].unwrap_or(0);
    lr.x[21] = regs[Aarch64::X21].unwrap_or(0);
    lr.x[22] = regs[Aarch64::X22].unwrap_or(0);
    lr.x[23] = regs[Aarch64::X23].unwrap_or(0);
    lr.x[24] = regs[Aarch64::X24].unwrap_or(0);
    lr.x[25] = regs[Aarch64::X25].unwrap_or(0);
    lr.x[26] = regs[Aarch64::X26].unwrap_or(0);
    lr.x[27] = regs[Aarch64::X27].unwrap_or(0);
    lr.x[28] = regs[Aarch64::X28].unwrap_or(0);

    if from_exception {
        unwind_lander_from_exception(&lr);
    } else {
        unwind_lander(&lr);
    }
}

#[repr(C)]
pub struct LandingRegs {
    pub x: [u64; 29], // x0 - x28
    pub fp: u64,      // x29
    pub lr: u64,      // x30
    pub sp: u64,      // x31
}

#[repr(C)]
pub struct CalleeSavedRegs {
    pub r: [u64; 11],
    // x19 - x29
    pub lr: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct Aarch64;

registers!(Aarch64, {
    X0 = (0, "X0"),
    X1 = (1, "X1"),
    X2 = (2, "X2"),
    X3 = (3, "X3"),
    X4 = (4, "X4"),
    X5 = (5, "X5"),
    X6 = (6, "X6"),
    X7 = (7, "X7"),
    X8 = (8, "X8"),
    X9 = (9, "X9"),
    X10 = (10, "X10"),
    X11 = (11, "X11"),
    X12 = (12, "X12"),
    X13 = (13, "X13"),
    X14 = (14, "X14"),
    X15 = (15, "X15"),
    X16 = (16, "X16"),
    X17 = (17, "X17"),
    X18 = (18, "X18"),
    X19 = (19, "X19"),
    X20 = (20, "X20"),
    X21 = (21, "X21"),
    X22 = (22, "X22"),
    X23 = (23, "X23"),
    X24 = (24, "X24"),
    X25 = (25, "X25"),
    X26 = (26, "X26"),
    X27 = (27, "X27"),
    X28 = (28, "X28"),
    X29 = (29, "X29"),
    X30 = (30, "X30"),
    SP = (31, "SP"),
});

pub const REG_RETURN_ADDRESS: Register = Aarch64::X30;
pub const REG_STACK_POINTER: Register = Aarch64::SP;
pub const REG_ARGUMENT: Register = Aarch64::X0;
