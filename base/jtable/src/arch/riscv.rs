// #[macro_export]
// macro_rules! arch_static_branch {
//     ($name:ident, $branch:ident) => {
//         unsafe{
//             core::arch::asm!(
//                 "
//                 .pushsection __jump_table, \"aw\"
//                 .align 3
//                 .quad 1, {target}, {name} + {branch}
//                 .popsection
//                 ",
//                 target = sym xxx,
//                 name = sym $name,
//                 branch = const $branch,
//             );
//             paste!{
//                 [<$name _is_false>]()
//             }
//         }
//     };
// }

use core::arch::riscv64::fence_i;

use crate::StaticKey;

const INST_FALSE: u16 = 0b_010_0_01010_00000_01; // 0x4501
const INST_TRUE: u16 = 0b_010_0_01010_00001_01; // 0x4505

#[macro_export]
macro_rules! define_static_key_true {
    ($name:ident) => {
        static $name: StaticKeyTrue = StaticKeyTrue::new();
        paste! {
            #[naked_function::naked]
            pub unsafe extern "C" fn [<$name _is_false>]() -> bool {
                asm!(
                    ".2byte 0x4505", // li a0,1
                    ".2byte 0x8082", // ret
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
                    ".2byte 0x4501",
                    ".2byte 0x8082",
                )
            }
        }
    };
}

#[inline(always)]
pub fn static_key_enable(key: &StaticKey, func_ptr: usize) {
    if key.is_enabled() {
        return;
    }
    key.set_enabled(true);
    // update the code
    unsafe {
        let func_start = func_ptr as *mut u16;
        func_start.write_volatile(INST_TRUE);
        fence_i();
    }
}

#[inline(always)]
pub fn static_key_disable(key: &StaticKey, func_ptr: usize) {
    if !key.is_enabled() {
        return;
    }
    key.set_enabled(false);
    // update the code
    unsafe {
        let func_start = func_ptr as *mut u16;
        func_start.write_volatile(INST_FALSE);
        fence_i();
    }
}
