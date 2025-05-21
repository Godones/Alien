//! panic 处理

use alloc::boxed::Box;
use core::{ffi::c_void, panic::PanicInfo, sync::atomic::AtomicBool};

use constants::{AlienError, AlienResult};
use ksym::lookup_kallsyms;
use platform::{println, println_color, system_shutdown};
use unwinding::abi::{UnwindContext, UnwindReasonCode, _Unwind_Backtrace, _Unwind_GetIP};

/// 递归标志
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

/// 错误处理
///
/// 发生 panic 是进行结果处理.目前我们会读取符号表信息，进行堆栈回溯
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
        data.counter += 1;
        let pc = _Unwind_GetIP(unwind_ctx);
        if pc > 0 {
            let fmt_str = unsafe { lookup_kallsyms(pc as u64, data.counter as i32) };
            println!("{}", fmt_str);
            if fmt_str.contains("main") {
                data.kernel_main = true;
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
