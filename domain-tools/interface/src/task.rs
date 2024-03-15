use crate::{Basic, InodeId};
use rref::{RRef, RpcResult};

pub trait TaskDomain: Basic {
    fn run(&self);
    fn trap_frame_virt_addr(&self) -> RpcResult<usize>;
    fn current_task_satp(&self) -> RpcResult<usize>;
    fn trap_frame_phy_addr(&self) -> RpcResult<usize>;
    fn heap_info(&self, tmp_heap_info: RRef<TmpHeapInfo>) -> RpcResult<RRef<TmpHeapInfo>>;
    fn brk(&self, addr: usize) -> RpcResult<isize>;
    fn get_fd(&self, fd: usize) -> RpcResult<InodeId>;
    fn copy_to_user(&self, src: *const u8, dst: *mut u8, len: usize) -> RpcResult<()>;
    fn copy_from_user(&self, src: *const u8, dst: *mut u8, len: usize) -> RpcResult<()>;
}

#[derive(Debug, Default)]
pub struct TmpHeapInfo {
    pub start: usize,
    pub current: usize,
}
