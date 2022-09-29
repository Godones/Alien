use crate::sbi::shutdown;
use core::panic::PanicInfo;

/// 错误处理
/// 
/// 发生 panic 是进行结果处理
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    error!("panic: {}",info.message().unwrap());
    shutdown()
}

/// 终止程序
/// 
/// abort
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!("abort()")
}
