#![no_std]

extern crate alloc;

use alloc::sync::Arc;
use interface::{Basic, DevicesDomain};

mod block;
mod gpu;
mod rtc;
mod uart;

type DeviceId = u64;
#[derive(Debug)]
pub struct DevicesDomainImpl;

impl DevicesDomainImpl {
    pub fn new() -> Self {
        Self
    }
}

impl Basic for DevicesDomainImpl {}

impl DevicesDomain for DevicesDomainImpl {}

fn main() -> Arc<dyn DevicesDomain> {
    let uart = libsyscall::get_uart_domain().unwrap();
    let gpu = libsyscall::get_gpu_domain().unwrap();
    let mouse = libsyscall::get_input_domain("mouse").unwrap();
    let keyboard = libsyscall::get_input_domain("keyboard").unwrap();
    let blk = libsyscall::get_cache_blk_domain().unwrap();
    let rtc = libsyscall::get_rtc_domain().unwrap();

    block::init_block_device(blk);
    gpu::init_gpu(gpu);
    rtc::init_rtc(rtc);
    uart::init_uart(uart);
    Arc::new(DevicesDomainImpl::new())
}
