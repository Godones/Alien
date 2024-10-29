use alloc::boxed::Box;
use core::panic::PanicInfo;

use platform::system_shutdown;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(p) = info.location() {
        println!("line {}, file {}: {}", p.line(), p.file(), info.message());
    } else {
        println!("no location information available");
    }
    #[cfg(feature = "rust-unwind")]
    {
        unwinding::panic::begin_panic(Box::new(()));
        // drop(guard);
    }
    system_shutdown();
}

#[cfg(feature = "test")]
pub fn test_unwind() {
    struct UnwindTest;
    impl Drop for UnwindTest {
        fn drop(&mut self) {
            println!("Drop UnwindTest");
        }
    }
    let res1 = unwinding::panic::catch_unwind(|| {
        let _unwind_test = UnwindTest;
        println!("Test panic...");
        panic!("Test panic");
    });
    assert_eq!(res1.is_err(), true);
    let res2 = unwinding::panic::catch_unwind(|| {
        let _unwind_test = UnwindTest;
        println!("Test no panic...");
        0
    });
    assert_eq!(res2.is_ok(), true);
}
