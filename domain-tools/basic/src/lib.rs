#![no_std]

#[macro_use]
pub mod console;
pub mod arch;
pub mod bus;
pub mod config;
pub mod io;
pub mod logging;
pub mod sync;
pub mod task;
pub mod time;
pub mod vm;

extern crate alloc;

pub use corelib::{backtrace, blk_crash_trick, get_domain, write_console};
pub use corelib::{check_kernel_space, kernel_satp, switch_task, trap_from_user, trap_to_user};
