use constants::AlienResult;
use rref::RRef;

use crate::{vfs::InodeId, Basic};
pub trait TaskDomain: Basic {
    fn init(&self) -> AlienResult<()>;
    fn run(&self);
    fn trap_frame_virt_addr(&self) -> AlienResult<usize>;
    fn current_task_satp(&self) -> AlienResult<usize>;
    fn trap_frame_phy_addr(&self) -> AlienResult<usize>;
    fn heap_info(&self, tmp_heap_info: RRef<TmpHeapInfo>) -> AlienResult<RRef<TmpHeapInfo>>;
    fn get_fd(&self, fd: usize) -> AlienResult<InodeId>;
    fn copy_to_user(&self, src: *const u8, dst: *mut u8, len: usize) -> AlienResult<()>;
    fn copy_from_user(&self, src: *const u8, dst: *mut u8, len: usize) -> AlienResult<()>;
    fn current_tid(&self) -> AlienResult<usize>;
    fn current_pid(&self) -> AlienResult<usize>;
    fn current_ppid(&self) -> AlienResult<usize>;
    /// Set current task to wait and switch to next task
    fn current_to_wait(&self) -> AlienResult<()>;
    fn wake_up_wait_task(&self, tid: usize) -> AlienResult<()>;
    fn do_brk(&self, addr: usize) -> AlienResult<isize>;
    fn do_clone(
        &self,
        flags: usize,
        stack: usize,
        ptid: usize,
        tls: usize,
        ctid: usize,
    ) -> AlienResult<isize>;
    fn do_wait4(
        &self,
        pid: isize,
        exit_code_ptr: usize,
        options: u32,
        _rusage: usize,
    ) -> AlienResult<isize>;
    fn do_execve(
        &self,
        filename_ptr: usize,
        argv_ptr: usize,
        envp_ptr: usize,
    ) -> AlienResult<isize>;
    fn do_yield(&self) -> AlienResult<isize>;
}

#[derive(Debug, Default)]
pub struct TmpHeapInfo {
    pub start: usize,
    pub current: usize,
}
