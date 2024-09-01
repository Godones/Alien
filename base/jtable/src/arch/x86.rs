use crate::StaticKey;

const INST_FALSE: [u8; 7] = [0x48, 0xc7, 0xc0, 0x00, 0x00, 0x00, 0x00]; // mov rax, 0
const INST_TRUE: [u8; 7] = [0x48, 0xc7, 0xc0, 0x01, 0x00, 0x00, 0x00]; // mov rax, 1

#[macro_export]
macro_rules! define_static_key_true {
    ($name:ident) => {
        static $name: StaticKeyTrue = StaticKeyTrue::new();
        paste! {
            #[naked_function::naked]
            pub unsafe extern "C" fn [<$name _is_false>]() -> bool {
                asm!(
                    ".byte 0x48",
                    ".byte 0xc7",
                    ".byte 0xc0",
                    ".byte 0x01",
                    ".byte 0x00",
                    ".byte 0x00",
                    ".byte 0x00",
                    ".byte 0xc3",
                )
            }
        }
    };
}

#[macro_export]
macro_rules! define_static_key_false {
    ($name:ident) => {
        static $name: StaticKeyFalse = StaticKeyFalse::new();
        paste! {
            #[naked_function::naked]
            pub unsafe extern "C" fn [<$name _is_false>]() -> bool {
                asm!(
                    ".byte 0x48",
                    ".byte 0xc7",
                    ".byte 0xc0",
                    ".byte 0x00",
                    ".byte 0x00",
                    ".byte 0x00",
                    ".byte 0x00",
                    ".byte 0xc3",
                )
            }
        }
    };
}

#[inline(always)]
pub fn static_key_enable<const T: bool>(key: &StaticKey<T>, func_ptr: usize) {
    if key.is_enabled() {
        return;
    }
    key.set_enabled(true);
    // update the code
    unsafe {
        let func_start = func_ptr as *mut u8;
        func_start.copy_from(INST_TRUE.as_ptr(), 7);
        core::arch::x86_64::_mm_mfence();
    }
}

#[inline(always)]
pub fn static_key_disable<const T: bool>(key: &StaticKey<T>, func_ptr: usize) {
    if !key.is_enabled() {
        return;
    }
    key.set_enabled(false);
    // update the code
    unsafe {
        let func_start = func_ptr as *mut u8;
        func_start.copy_from(INST_FALSE.as_ptr(), 7);
        core::arch::x86_64::_mm_mfence();
    }
}
