use crate::sbi::shutdown;
use core::panic::PanicInfo;

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
    shutdown()
}
