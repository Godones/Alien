#![allow(unused)]
use core::arch::asm;

const SBI_CONSOLE_PUT_CHAR: usize = 1;
const SBI_CONSOLE_GET_CHAR: usize = 2;
const SBI_SHUTDOWN: usize = 8;

fn sbi_call(which: usize, arg0: usize, arg1: usize, arg2: usize) -> i32 {
    let mut ret;
    unsafe {
        asm!("ecall",
        in("a7") which,
        inlateout("a0") arg0 as i32 => ret,
        in("a1") arg1,
        in("a2") arg2);
    }
    ret
}

pub fn set_timer(time: usize) {
    sbi_call_5(EXTENSION_TIMER, 0, [time, 0, 0, 0, 0]);
}

pub fn system_shutdown() -> ! {
    loop {
        system_reset(0, 1);
    }
}

pub fn console_putchar(ch: u8) {
    sbi_call(SBI_CONSOLE_PUT_CHAR, ch as usize, 0, 0);
}

pub fn console_getchar() -> char {
    sbi_call(SBI_CONSOLE_GET_CHAR, 0, 0, 0) as u8 as char
}

#[repr(C)]
#[derive(Debug)]
pub struct SbiRet {
    /// Error number
    pub error: isize,
    /// Result value
    pub value: isize,
}

/// SBI basic extension
pub const EXTENSION_BASE: usize = 0x10;
/// SBI timer extension
pub const EXTENSION_TIMER: usize = 0x54494D45;
/// SBI IPI extension
pub const EXTENSION_IPI: usize = 0x735049;
/// SBI RFENCE extension
pub const EXTENSION_RFENCE: usize = 0x52464E43;
/// SBI HSM extension
pub const EXTENSION_HSM: usize = 0x48534D;
// pub const EXTENSION_SRST: usize = 0x53525354;
const FUNCTION_HSM_HART_START: usize = 0x0;
// const FUNCTION_HSM_HART_STOP: usize = 0x1;
// const FUNCTION_HSM_HART_GET_STATUS: usize = 0x2;
const FUNCTION_HSM_HART_SUSPEND: usize = 0x3;

/// System Reset Extension
pub const EXTENSION_SRST: usize = 0x53525354;

#[inline(always)]
fn sbi_call_5(extension: usize, function: usize, args: [usize; 5]) -> SbiRet {
    let (error, value);
    unsafe {
        asm!(
            "ecall",
            in("a0") args[0],
            in("a1") args[1],
            in("a2") args[2],
            in("a3") args[3],
            in("a4") args[4],
            in("a6") function,
            in("a7") extension,
            lateout("a0") error,
            lateout("a1") value,
        )
    }
    SbiRet { error, value }
}

pub fn system_reset(ty: u32, reason: u32) -> SbiRet {
    sbi_call_5(EXTENSION_SRST, 0, [ty as usize, reason as usize, 0, 0, 0])
}

pub fn hart_suspend(suspend_type: u32, resume_addr: usize, opaque: usize) -> SbiRet {
    sbi_call_5(
        EXTENSION_HSM,
        FUNCTION_HSM_HART_SUSPEND,
        [suspend_type as usize, resume_addr, opaque, 0, 0],
    )
}

pub fn hart_start(hart_id: usize, start_addr: usize, opaque: usize) -> SbiRet {
    sbi_call_5(
        EXTENSION_HSM,
        FUNCTION_HSM_HART_START,
        [hart_id, start_addr, opaque, 0, 0],
    )
}

pub fn send_ipi(hart_mask: usize, hart_mask_base: usize) -> SbiRet {
    sbi_call_5(EXTENSION_IPI, 0, [hart_mask, hart_mask_base, 0, 0, 0])
}

pub fn remote_fence_i(hart_mask: usize, hart_mask_base: usize) -> SbiRet {
    // Remote FENCE.I (FID #0)
    sbi_call_5(EXTENSION_RFENCE, 0, [hart_mask, hart_mask_base, 0, 0, 0])
}

/// Any function wishes to use range of addresses (i.e. start_addr and size), have to abide by the below
/// constraints on range parameters.
/// The remote fence function acts as a full TLB flush if
/// - start_addr and size are both 0
/// - size is equal to 2^XLEN-1
pub fn remote_sfence_vma(
    hart_mask: usize,
    hart_mask_base: usize,
    start_addr: usize,
    size: usize,
) -> SbiRet {
    // Remote SFENCE.VMA (FID #1)
    sbi_call_5(
        EXTENSION_RFENCE,
        1,
        [hart_mask, hart_mask_base, start_addr, size, 0],
    )
}

// pub fn remote_sfence_vma_asid(hart_mask:usize, hart_mask_base:usize, start_addr:usize, size:usize, asid:usize)->SbiRet{
//     // Remote SFENCE.VMA.ASID (FID #2)
//     sbi_call_5(EXTENSION_RFENCE, 2, [hart_mask, hart_mask_base, start_addr, size, asid])
// }
//
// pub fn remote_hfence_gvma_vmid(hart_mask:usize, hart_mask_base:usize, start_addr:usize, size:usize, vmid:usize)->SbiRet{
//     // Remote HFENCE.GVMA (FID #3)
//     sbi_call_5(EXTENSION_RFENCE, 3, [hart_mask, hart_mask_base, start_addr,size,vmid])
// }
//
// pub fn remote_hfence_gvma(hart_mask:usize,hart_mask_base:usize, start_addr:usize, size:usize)->SbiRet{
//     //: Remote HFENCE.GVMA (FID #4)
//     sbi_call_5(EXTENSION_RFENCE, 4, [hart_mask, hart_mask_base, start_addr,size,0])
// }
//
// pub fn remote_hfence_vvma_asid(hart_mask:usize, hart_mask_base:usize, start_addr:usize, size:usize, asid:usize)->SbiRet{
//     // Remote HFENCE.VVMA.ASID (FID #5)
//     sbi_call_5(EXTENSION_RFENCE, 5, [hart_mask, hart_mask_base, start_addr,size,asid])
// }
//
// pub fn remote_hfence_vvma(hart_mask:usize, hart_mask_base:usize, start_addr:usize, size:usize)->SbiRet{
//     // Remote HFENCE.VVMA (FID #6)
//     sbi_call_5(EXTENSION_RFENCE, 6, [hart_mask, hart_mask_base, start_addr,size,0])
// }
