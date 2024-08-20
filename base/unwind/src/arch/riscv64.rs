use core::arch::asm;

use gimli::Register;

use crate::{registers, registers::Registers};

extern "C" {
    pub fn unwind_lander(regs: *const LandingRegs) -> !;
    pub fn unwind_trampoline(ctx: usize);
}

#[inline(always)]
pub fn hart_id() -> usize {
    let mut id: usize;
    unsafe {
        asm!(
        "mv {},tp", out(reg)id,
        );
    }
    // lower 32 bits of tp register is the hart id
    id as u32 as usize
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
unsafe extern "C" fn unwind_recorder(
    ctx: *mut super::UnwindingContext,
    stack: u64,
    saved_regs: *mut CalleeSavedRegs,
) {
    let saved_regs = &*saved_regs;
    let mut registers = Registers::default();

    registers[Riscv64::X8] = Some(saved_regs.s0);
    registers[Riscv64::X9] = Some(saved_regs.s1);
    registers[Riscv64::X18] = Some(saved_regs.r[0]);
    registers[Riscv64::X19] = Some(saved_regs.r[1]);
    registers[Riscv64::X20] = Some(saved_regs.r[2]);
    registers[Riscv64::X21] = Some(saved_regs.r[3]);
    registers[Riscv64::X22] = Some(saved_regs.r[4]);
    registers[Riscv64::X23] = Some(saved_regs.r[5]);
    registers[Riscv64::X24] = Some(saved_regs.r[6]);
    registers[Riscv64::X25] = Some(saved_regs.r[7]);
    registers[Riscv64::X26] = Some(saved_regs.r[8]);
    registers[Riscv64::X27] = Some(saved_regs.r[9]);
    registers[Riscv64::X2] = Some(stack);
    registers[Riscv64::X1] = Some(saved_regs.ra);
    registers[Riscv64::X4] = Some(hart_id() as _);

    super::unwind_from_panic_stub(registers, ctx);
}

pub unsafe fn land(regs: &Registers, landing_pad_address: u64, _from_exception: bool) {
    let mut lr = LandingRegs::default();
    lr.x[0] = regs[Riscv64::X0].unwrap_or(0);
    lr.x[1] = landing_pad_address;
    lr.x[2] = regs[Riscv64::X2].unwrap_or(0);
    lr.x[3] = regs[Riscv64::X3].unwrap_or(0);
    lr.x[4] = regs[Riscv64::X4].unwrap_or(0);
    lr.x[5] = regs[Riscv64::X5].unwrap_or(0);
    lr.x[6] = regs[Riscv64::X6].unwrap_or(0);
    lr.x[7] = regs[Riscv64::X7].unwrap_or(0);
    lr.x[8] = regs[Riscv64::X8].unwrap_or(0);
    lr.x[9] = regs[Riscv64::X9].unwrap_or(0);
    lr.x[10] = regs[Riscv64::X10].unwrap_or(0);
    lr.x[11] = regs[Riscv64::X11].unwrap_or(0);
    lr.x[12] = regs[Riscv64::X12].unwrap_or(0);
    lr.x[13] = regs[Riscv64::X13].unwrap_or(0);
    lr.x[14] = regs[Riscv64::X14].unwrap_or(0);
    lr.x[15] = regs[Riscv64::X15].unwrap_or(0);
    lr.x[16] = regs[Riscv64::X16].unwrap_or(0);
    lr.x[17] = regs[Riscv64::X17].unwrap_or(0);
    lr.x[18] = regs[Riscv64::X18].unwrap_or(0);
    lr.x[19] = regs[Riscv64::X19].unwrap_or(0);
    lr.x[20] = regs[Riscv64::X20].unwrap_or(0);
    lr.x[21] = regs[Riscv64::X21].unwrap_or(0);
    lr.x[22] = regs[Riscv64::X22].unwrap_or(0);
    lr.x[23] = regs[Riscv64::X23].unwrap_or(0);
    lr.x[24] = regs[Riscv64::X24].unwrap_or(0);
    lr.x[25] = regs[Riscv64::X25].unwrap_or(0);
    lr.x[26] = regs[Riscv64::X26].unwrap_or(0);
    lr.x[27] = regs[Riscv64::X27].unwrap_or(0);
    lr.x[28] = regs[Riscv64::X28].unwrap_or(0);
    lr.x[29] = regs[Riscv64::X29].unwrap_or(0);
    lr.x[30] = regs[Riscv64::X30].unwrap_or(0);
    lr.x[31] = regs[Riscv64::X31].unwrap_or(0);

    unwind_lander(&lr);
}

core::arch::global_asm! {
r#"
.global unwind_trampoline
unwind_trampoline:
.cfi_startproc
     mv a1, sp
     addi sp, sp, -0x70
     .cfi_adjust_cfa_offset 0x70
     sd x8,  0x00(sp)
     sd x9,  0x08(sp)
     sd x18, 0x10(sp)
     sd x19, 0x18(sp)
     sd x20, 0x20(sp)
     sd x21, 0x28(sp)
     sd x22, 0x30(sp)
     sd x23, 0x38(sp)
     sd x24, 0x40(sp)
     sd x25, 0x48(sp)
     sd x26, 0x50(sp)
     sd x27, 0x58(sp)
     sd ra,  0x60(sp)
     .cfi_rel_offset ra, 0x60
     mv a2, sp
     jal unwind_recorder
     ld ra,  0x60(sp)
     .cfi_restore ra
     addi sp, sp, 0x70
     .cfi_adjust_cfa_offset -0x70
     ret
.cfi_endproc
.global unwind_lander
unwind_lander:
    ld x1, 0x08(a0)
    ld x2, 0x10(a0)
    ld x3, 0x18(a0)
    ld x4, 0x20(a0)
    ld x5, 0x28(a0)
    ld x6, 0x30(a0)
    ld x7, 0x38(a0)
    ld x8, 0x40(a0)
    ld x9, 0x48(a0)
    // skip x10(a0)
    ld x11, 0x58(a0)
    ld x12, 0x60(a0)
    ld x13, 0x68(a0)
    ld x14, 0x70(a0)
    ld x15, 0x78(a0)
    ld x16, 0x80(a0)
    ld x17, 0x88(a0)
    ld x18, 0x90(a0)
    ld x19, 0x98(a0)
    ld x20, 0xa0(a0)
    ld x21, 0xa8(a0)
    ld x22, 0xb0(a0)
    ld x23, 0xb8(a0)
    ld x24, 0xc0(a0)
    ld x25, 0xc8(a0)
    ld x26, 0xd0(a0)
    ld x27, 0xd8(a0)
    ld x28, 0xe0(a0)
    ld x29, 0xe8(a0)
    ld x30, 0xf0(a0)
    ld x31, 0xf8(a0)
    ld a0, 0x50(a0)
    ret
"#
}

#[repr(C)]
#[derive(Default)]
pub struct LandingRegs {
    pub x: [u64; 32], // x0 - x31
}

#[repr(C)]
pub struct CalleeSavedRegs {
    pub s0: u64,
    // x8
    pub s1: u64,
    // x9
    pub r: [u64; 10],
    // x18 - x27
    pub ra: u64, // x1
}

#[derive(Debug, Clone, Copy)]
pub struct Riscv64;

registers!(Riscv64, {
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
    X31 = (31, "X31"),
});

pub const REG_RETURN_ADDRESS: Register = Riscv64::X1;
pub const REG_STACK_POINTER: Register = Riscv64::X2;
pub const REG_ARGUMENT: Register = Riscv64::X10;
