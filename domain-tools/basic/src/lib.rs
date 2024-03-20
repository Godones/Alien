#![no_std]

#[macro_use]
pub mod console;
pub mod arch;
pub mod frame;
pub mod logging;
pub mod time;

extern crate alloc;

pub use corelib::{backtrace, blk_crash_trick, get_domain, write_console};
pub use corelib::{
    check_kernel_space, kernel_satp, switch_task, trampoline_addr, trap_from_user, trap_to_user,
};
