use crate::config::KERNEL_STACK_SIZE;
use crate::memory::{kernel_satp, KERNEL_SPACE};
use crate::task::context::ThreadContext;
use crate::task::process::Process;
use crate::task::stack::Stack;
use crate::trap::{kernel_trap_vector, TrapFrame};
use alloc::sync::{Arc, Weak};
use spin::Mutex;

pub struct Thread {
    pub process: Weak<Process>,
    pub kernel_stack: Stack,
    pub inner: Arc<Mutex<ThreadInner>>,
}

pub struct ThreadInner {
    pub exit_code: i32,
    pub tid: u32,
    pub trap_frame: TrapFrame,
    pub state: ThreadState,
    pub context: ThreadContext,
}

#[derive(Debug)]
pub enum ThreadState {
    Init,
    Running,
    Waiting,
    Dead,
    Sleeping,
}

impl Thread {
    /// 创建一个空线程
    pub fn empty() -> Self {
        let kernel_stack = Stack::new(KERNEL_STACK_SIZE).expect("No space for KernelStack");
        let thread_inner = ThreadInner {
            exit_code: 0,
            tid: 0,
            trap_frame: TrapFrame::empty(),
            state: ThreadState::Init,
            context: ThreadContext::default(),
        };
        let thread = Self {
            process: Weak::new(),
            kernel_stack,
            inner: Arc::new(Mutex::new(thread_inner)),
        };
        thread
    }
}

/// 创建内核线程
///
/// 内核线程只会在内核态运行，不访问用户态资源
///
/// func:函数地址
///
pub fn create_kernel_thread(func: usize) {
    let thread = Thread::empty();
    // 设置内核线程的属性
    // 内核线程的栈帧
    let mut thread_inner = thread.inner.lock();
    thread_inner.trap_frame = TrapFrame {
        x: [0; 32],
        sepc: func,
        k_satp: kernel_satp(),
        k_sp: thread.kernel_stack.top(),
        trap_handler: 0,
        hart_id: 0,
    }
}
