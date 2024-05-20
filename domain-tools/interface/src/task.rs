use downcast_rs::{impl_downcast, DowncastSync};
use gproxy::proxy;
use pod::Pod;
use rref::{RRef, RRefVec};

use super::AlienResult;
use crate::{vfs::InodeID, Basic};
#[proxy(TaskDomainProxy)]
pub trait TaskDomain: Basic + DowncastSync {
    fn init(&self) -> AlienResult<()>;
    // fn run(&self) -> AlienResult<()>;
    fn trap_frame_virt_addr(&self) -> AlienResult<usize>;
    fn current_task_satp(&self) -> AlienResult<usize>;
    fn trap_frame_phy_addr(&self) -> AlienResult<usize>;
    fn heap_info(&self, tmp_heap_info: RRef<TmpHeapInfo>) -> AlienResult<RRef<TmpHeapInfo>>;
    fn get_fd(&self, fd: usize) -> AlienResult<InodeID>;
    fn add_fd(&self, inode: InodeID) -> AlienResult<usize>;
    fn remove_fd(&self, fd: usize) -> AlienResult<InodeID>;
    fn fs_info(&self) -> AlienResult<(InodeID, InodeID)>;
    fn set_cwd(&self, inode: InodeID) -> AlienResult<()>;
    fn copy_to_user(&self, dst: usize, buf: &[u8]) -> AlienResult<()>;
    fn copy_from_user(&self, src: usize, buf: &mut [u8]) -> AlienResult<()>;
    fn read_string_from_user(
        &self,
        src: usize,
        buf: RRefVec<u8>,
    ) -> AlienResult<(RRefVec<u8>, usize)>;
    fn current_tid(&self) -> AlienResult<usize>;
    fn current_pid(&self) -> AlienResult<usize>;
    fn current_ppid(&self) -> AlienResult<usize>;
    /// Set current task to wait and switch to next task
    // fn current_to_wait(&self) -> AlienResult<()>;
    // fn wake_up_wait_task(&self, tid: usize) -> AlienResult<()>;
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
    // fn do_yield(&self) -> AlienResult<isize>;
    fn do_set_tid_address(&self, tidptr: usize) -> AlienResult<isize>;
    fn do_mmap(
        &self,
        start: usize,
        len: usize,
        prot: u32,
        flags: u32,
        fd: usize,
        offset: usize,
    ) -> AlienResult<isize>;
    fn do_munmap(&self, start: usize, len: usize) -> AlienResult<isize>;
    fn do_sigaction(&self, signum: u8, act: usize, oldact: usize) -> AlienResult<isize>;
    fn do_sigprocmask(&self, how: usize, set: usize, oldset: usize) -> AlienResult<isize>;
    fn do_fcntl(&self, fd: usize, cmd: usize) -> AlienResult<(InodeID, usize)>;
    fn do_prlimit(
        &self,
        pid: usize,
        resource: usize,
        new_limit: usize,
        old_limit: usize,
    ) -> AlienResult<isize>;
    fn do_dup(&self, old_fd: usize, new_fd: Option<usize>) -> AlienResult<isize>;
    fn do_pipe2(&self, r: InodeID, w: InodeID, pipe: usize) -> AlienResult<isize>;
    fn do_exit(&self, exit_code: isize) -> AlienResult<isize>;
}

#[derive(Debug, Default)]
pub struct TmpHeapInfo {
    pub start: usize,
    pub current: usize,
}

impl dyn TaskDomain {
    pub fn read_val_from_user<T: Pod>(&self, src: usize) -> AlienResult<T> {
        let mut val = T::new_uninit();
        self.copy_from_user(src, val.as_bytes_mut())?;
        Ok(val)
    }

    pub fn write_val_to_user<T: Pod>(&self, dst: usize, val: &T) -> AlienResult<()> {
        self.copy_to_user(dst, val.as_bytes())
    }
}

impl_downcast!(sync TaskDomain);
