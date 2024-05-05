extern crate tinyrlibc;
use core::{ffi::c_void, fmt::Write};

use cty::{c_char, c_int, size_t};

#[no_mangle]
unsafe extern "C" fn printf(str: *const c_char, mut args: ...) -> c_int {
    use printf_compat::{format, output};
    // let mut s = String::new();
    let bytes_written = format(str, args.as_va_list(), output::fmt_write(&mut FakeOut));
    // println!("{}", s);
    bytes_written
}

struct FakeOut;
impl Write for FakeOut {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        print!("{}", s);
        Ok(())
    }
}

#[no_mangle]
static stdout: usize = 0;

#[no_mangle]
extern "C" fn fflush(file: *mut c_void) -> c_int {
    assert!(file.is_null());
    0
}

#[no_mangle]
unsafe extern "C" fn qsort(
    base: *mut c_void,
    nmemb: size_t,
    width: size_t,
    compar: Option<unsafe extern "C" fn(*const c_void, *const c_void) -> c_int>,
) {
    let compar = compar.unwrap();

    if nmemb <= 1 {
        return;
    }

    let base = base.cast::<u8>();
    let mut gap = nmemb;

    loop {
        gap = next_gap(gap);
        let mut any_swapped = false;
        let mut a = base;
        let mut b = base.add(gap * width);
        for _ in 0..nmemb - gap {
            if compar(a.cast(), b.cast()) > 0 {
                swap(a, b, width);
                any_swapped = true;
            }
            a = a.add(width);
            b = b.add(width);
        }

        if gap <= 1 && !any_swapped {
            break;
        }
    }
}

fn next_gap(gap: size_t) -> size_t {
    let gap = (gap * 10) / 13;

    if gap == 9 || gap == 10 {
        11 // apply the "rule of 11"
    } else if gap <= 1 {
        1
    } else {
        gap
    }
}

unsafe fn swap(a: *mut u8, b: *mut u8, width: size_t) {
    for i in 0..width {
        core::ptr::swap(a.add(i), b.add(i));
    }
}
