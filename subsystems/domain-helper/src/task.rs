use alloc::sync::Arc;
use alloc::vec::Vec;
use libsyscall::{KTask, KTaskShim};

pub struct TaskShimImpl;

#[allow(unused)]
impl KTaskShim for TaskShimImpl {
    fn get_task(&self) -> Arc<dyn KTask> {
        todo!()
    }

    fn put_task(&self, task: Arc<dyn KTask>) {
        todo!()
    }

    fn suspend(&self) {
        todo!()
    }

    fn transfer_ptr_raw(&self, ptr: usize) -> usize {
        todo!()
    }

    fn transfer_buf_raw(&self, src: usize, size: usize) -> Vec<&mut [u8]> {
        todo!()
    }
}
