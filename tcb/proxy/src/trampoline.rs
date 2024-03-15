use continuation::{register_continuation, Continuation};

#[no_mangle]
pub extern "C" fn register_cont(cont: &Continuation) {
    register_continuation(cont)
}
