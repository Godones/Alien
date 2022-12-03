use core::arch::asm;

pub fn hart_id() -> usize {
    let id: usize;
    unsafe {
        asm!(
        "mv {},tp", out(reg)id
        );
    }
    id
}
