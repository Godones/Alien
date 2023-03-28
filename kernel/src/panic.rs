use crate::sbi::shutdown;
use core::panic::PanicInfo;
use core::sync::atomic::AtomicBool;

static RECURSION: AtomicBool = AtomicBool::new(false);

/// 错误处理
///
/// 发生 panic 是进行结果处理
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    if let Some(p) = info.location() {
        println!(
            "line {}, file {}: {}",
            p.line(),
            p.file(),
            info.message().unwrap()
        );
    } else {
        println!("no location information available");
    }
    if !RECURSION.load(core::sync::atomic::Ordering::SeqCst) {
        RECURSION.store(true, core::sync::atomic::Ordering::SeqCst);
        back_trace();
    }
    shutdown();
    loop {}
}

fn back_trace() {
    println!("---START BACKTRACE---");
    let info = crate::trace::init_kernel_trace();
    let func_info = unsafe { trace_lib::my_trace(info) };
    func_info.iter().for_each(|x| {
        println!("{}", x);
    });
    println!("---END   BACKTRACE---");
}
