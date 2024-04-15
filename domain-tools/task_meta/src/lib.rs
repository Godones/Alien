#![no_std]
#![forbid(unsafe_code)]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct TaskContext {
    /// ra
    ra: usize,
    /// sp
    sp: usize,
    /// s0 ~ s11
    s: [usize; 12],
}

impl TaskContext {
    pub fn new(ra: usize, sp: usize) -> Self {
        Self { ra, sp, s: [0; 12] }
    }

    pub const fn empty() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TaskMeta {
    pub context: TaskContext,
    // other fields
    pub tid: usize,
    pub statue: TaskStatus,
}

impl TaskMeta {
    pub fn new(tid: usize, context: TaskContext) -> Self {
        Self {
            tid,
            context,
            statue: TaskStatus::Ready,
        }
    }
    pub fn tid(&self) -> usize {
        self.tid
    }
    pub fn get_context_raw_ptr(&self) -> *mut TaskContext {
        &self.context as *const TaskContext as *mut _
    }
    pub fn get_context_raw_mut_ptr(&mut self) -> *mut TaskContext {
        &mut self.context as *mut TaskContext
    }
    pub fn set_status(&mut self, status: TaskStatus) {
        self.statue = status;
    }
    pub fn status(&self) -> TaskStatus {
        self.statue
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum TaskStatus {
    /// 就绪态
    Ready,
    /// 运行态
    Running,
    /// 等待一个事件
    Waiting,
    /// 僵尸态，等待父进程回收资源
    Zombie,
    /// 终止态
    Terminated,
}
