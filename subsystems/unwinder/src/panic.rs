//! panic 处理

use alloc::boxed::Box;
use core::{ffi::c_void, panic::PanicInfo, sync::atomic::AtomicBool};

use constants::{AlienError, AlienResult};
use platform::{println, println_color, system_shutdown};
use spin::Once;
use unwinding::abi::{UnwindContext, UnwindReasonCode, _Unwind_Backtrace, _Unwind_GetIP};

static RECURSION: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
struct PanicGuard;

impl PanicGuard {
    pub fn new() -> Self {
        arch::enbale_float();
        Self
    }
}

impl Drop for PanicGuard {
    fn drop(&mut self) {
        arch::disable_float();
    }
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    if let Some(p) = info.location() {
        println!("line {}, file {}: {}", p.line(), p.file(), info.message());
    } else {
        println!("no location information available");
    }
    if !RECURSION.swap(true, core::sync::atomic::Ordering::SeqCst) {
        if info.can_unwind() {
            let guard = Box::new(PanicGuard::new());
            print_stack_trace();
            let _res = unwinding::panic::begin_panic(guard);
            println_color!(31, "panic unreachable: {:?}", _res.0);
        }
    }
    println!("!TEST FINISH!");
    system_shutdown();
}

pub fn print_stack_trace() {
    println!("Rust Panic Backtrace:");
    struct CallbackData {
        counter: usize,
        kernel_main: bool,
    }
    extern "C" fn callback(unwind_ctx: &UnwindContext<'_>, arg: *mut c_void) -> UnwindReasonCode {
        let data = unsafe { &mut *(arg as *mut CallbackData) };
        if data.kernel_main {
            // If we are in kernel_main, we don't need to print the backtrace.
            return UnwindReasonCode::NORMAL_STOP;
        }
        let mut name_buf = [0u8; 1024];
        data.counter += 1;
        let pc = _Unwind_GetIP(unwind_ctx);
        if pc > 0 {
            let res = PANIC_HELPER
                .get()
                .and_then(|helper| helper.lookup_symbol(pc, &mut name_buf));
            if let Some((name, addr)) = res {
                println_color!(
                    33,
                    "  #{:<2} {:#018x} - {} (+{:#x})",
                    data.counter,
                    addr,
                    name,
                    pc as usize - addr
                );
            } else {
                println_color!(
                    33,
                    "  #{:<2} {:#018x} - <unknown>",
                    data.counter,
                    pc as usize
                );
            }
        }
        UnwindReasonCode::NO_REASON
    }
    let mut data = CallbackData {
        counter: 0,
        kernel_main: false,
    };
    _Unwind_Backtrace(callback, &mut data as *mut _ as _);
}

pub fn kernel_catch_unwind<R, F: FnOnce() -> R>(f: F) -> AlienResult<R> {
    let res = unwinding::panic::catch_unwind(f);
    match res {
        Ok(r) => Ok(r),
        Err(e) => {
            println_color!(31, "Catch Unwind Error: {:?}", e);
            Err(AlienError::EIO)
        }
    }
}

pub trait PanicHelper: Send + Sync {
    /// Looks up the symbol name and its base address by the given address.
    fn lookup_symbol<'a>(&self, addr: usize, buf: &'a mut [u8; 1024]) -> Option<(&'a str, usize)>;
}

static PANIC_HELPER: Once<&'static dyn PanicHelper> = Once::new();

/// Sets the panic helper.
pub fn set_panic_helper(helper: &'static dyn PanicHelper) {
    PANIC_HELPER.call_once(|| helper);
}
