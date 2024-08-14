use core::panic::PanicInfo;

use ksync::Mutex;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
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
    #[cfg(feature = "rust-unwind")]
    {
        let _guard = FAKE_LOCK.lock();
        unwind::unwind_from_panic(3);
        // drop(guard);
    }
}
static FAKE_LOCK: Mutex<()> = Mutex::new(());

#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {
    println_color!(31, "rust_eh_personality called");
    loop {}
}

#[allow(non_snake_case)]
#[no_mangle]
extern "C" fn _Unwind_Resume(arg: usize) -> ! {
    println_color!(31, "Unwind resume arg {:#x}", arg);
    unwind::unwind_resume(arg);
}

#[cfg(test)]
pub fn test_unwind() {
    struct UnwindTest;
    impl Drop for UnwindTest {
        fn drop(&mut self) {
            println!("Drop UnwindTest");
        }
    }
    let res1 = unwind::catch::catch_unwind(|| {
        let _unwind_test = UnwindTest;
        println!("Test panic...");
        panic!("Test panic");
    });
    assert_eq!(res1.is_err(), true);
    let res2 = unwind::catch::catch_unwind(|| {
        let _unwind_test = UnwindTest;
        println!("Test no panic...");
        0
    });
    assert_eq!(res2.is_ok(), true);
}
