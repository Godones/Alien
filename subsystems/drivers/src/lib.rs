#![no_std]
extern crate alloc;

use alloc::boxed::Box;
use alloc::sync::Arc;
use downcast_rs::{impl_downcast, DowncastSync};
use spin::Once;

pub mod block_device;
pub mod gpu;
pub mod hal;
pub mod input;
pub mod net;
pub mod rtc;
pub mod uart;

pub trait DriverTask: Send + Sync + DowncastSync {
    fn to_wait(&self);
    fn to_wakeup(&self);
    fn have_signal(&self) -> bool;
}

impl_downcast!(sync DriverTask);

pub trait DriverWithTask: Send + Sync {
    fn get_task(&self) -> Arc<dyn DriverTask>;
    fn put_task(&self, task: Arc<dyn DriverTask>);
    fn suspend(&self);
}

static DRIVER_TASK: Once<Box<dyn DriverWithTask>> = Once::new();

pub fn register_task_func(task_func: Box<dyn DriverWithTask>) {
    DRIVER_TASK.call_once(|| task_func);
}
