use crate::println;
use crate::syscall::sys_shutdown;
use core::sync::atomic::AtomicUsize;

static ERROR: AtomicUsize = AtomicUsize::new(0);

#[panic_handler]
fn panic_handler(panic_info: &core::panic::PanicInfo) -> ! {
    let err = panic_info.message().unwrap();
    let val = ERROR.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    if val != 0 {
        sys_shutdown();
        loop {}
    }
    if let Some(location) = panic_info.location() {
        println!(
            "Panicked at {}:{}, {}",
            location.file(),
            location.line(),
            err
        );
    } else {
        println!("Panicked: {}", err);
    }
    sys_shutdown();
    loop {}
}
