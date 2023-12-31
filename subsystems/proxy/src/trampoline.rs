use alloc::vec::Vec;
use continuation::Continuation;
use ksync::Mutex;
use spin::Lazy;

#[no_mangle]
pub extern "C" fn register_cont(cont: &Continuation) {
    register_continuation(cont)
}

/// Register a continuation for the current thread in the current domain.
pub fn register_continuation(context: &Continuation) {
    // info!("[register_continuation]: {:#x?}", context);
    let mut binding = TASK_CONTEXT.lock();
    let mut new_context = context.clone();
    new_context.regs[2] += 33 * 8; // sp += 33 * 8
    binding.push(new_context);
    if context.func != 0 {
        // platform::system_shutdown();
    }
}

pub fn pop_continuation() -> Option<Continuation> {
    let mut binding = TASK_CONTEXT.lock();
    binding.pop()
}

static TASK_CONTEXT: Lazy<Mutex<Vec<Continuation>>> = Lazy::new(|| Mutex::new(Vec::new()));
