use alloc::boxed::Box;
use core::{any::Any, mem::ManuallyDrop};

pub type PanicError = Box<dyn Any + Send>;

pub fn catch_unwind<F: FnOnce() -> R, R>(f: F) -> Result<R, PanicError> {
    unsafe { r#try(f) }
}

/// Invoke a closure, capturing the cause of an unwinding panic if one occurs.
pub unsafe fn r#try<R, F: FnOnce() -> R>(f: F) -> Result<R, PanicError> {
    union Data<F, R> {
        f: ManuallyDrop<F>,
        r: ManuallyDrop<R>,
        p: ManuallyDrop<PanicError>,
    }

    let mut data = Data {
        f: ManuallyDrop::new(f),
    };

    let data_ptr = &mut data as *mut _ as *mut u8;
    return if core::intrinsics::catch_unwind(do_call::<F, R>, data_ptr, do_catch::<F, R>) == 0 {
        Ok(ManuallyDrop::into_inner(data.r))
    } else {
        Err(ManuallyDrop::into_inner(data.p))
    };

    #[inline]
    fn do_call<F: FnOnce() -> R, R>(data: *mut u8) {
        unsafe {
            let data = data as *mut Data<F, R>;
            let data = &mut (*data);
            let f = ManuallyDrop::take(&mut data.f);
            data.r = ManuallyDrop::new(f());
        }
    }

    fn cleanup(payload: *mut u8) -> Box<super::UnwindingContext> {
        let obj = unsafe { Box::from_raw(payload as *mut super::UnwindingContext) };
        obj
    }

    #[inline]
    fn do_catch<F: FnOnce() -> R, R>(data: *mut u8, payload: *mut u8) {
        unsafe {
            let data = data as *mut Data<F, R>;
            let data = &mut (*data);
            if payload as usize == 0 {
                return;
            }
            let obj = cleanup(payload);
            data.p = ManuallyDrop::new(obj);
        }
    }
}
