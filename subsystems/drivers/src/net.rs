use core::ptr::NonNull;

use constants::time::TimeSpec;
pub use loopback::LoopbackDev;
use netcore::{KernelNetFunc, NetInstant};
use timer::TimeNow;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use virtio_net::VirtIONetDeviceWrapper;

use crate::hal::HalImpl;

pub const NET_BUFFER_LEN: usize = 4096;
pub const NET_QUEUE_SIZE: usize = 128;

pub struct VirtIONetDriver;

impl VirtIONetDriver {
    pub fn from_mmio(
        addr: usize,
    ) -> VirtIONetDeviceWrapper<HalImpl, MmioTransport, NET_QUEUE_SIZE> {
        let header = NonNull::new(addr as *mut VirtIOHeader).unwrap();
        let transport = unsafe { MmioTransport::new(header) }.unwrap();
        let device = VirtIONetDeviceWrapper::new(transport, NET_BUFFER_LEN);
        device
    }
}

#[derive(Debug, Default)]
pub struct NetNeedFunc;
#[derive(Debug, Default)]
pub struct NetNeedFuncEmpty;

impl KernelNetFunc for NetNeedFuncEmpty {
    fn now(&self) -> NetInstant {
        let time_spec = TimeSpec::now();
        NetInstant {
            micros: time_spec.tv_sec as i64 * 1000_000 + time_spec.tv_nsec as i64 / 1000,
        }
    }

    fn yield_now(&self) -> bool {
        false
    }
}

impl KernelNetFunc for NetNeedFunc {
    fn now(&self) -> NetInstant {
        let time_spec = TimeSpec::now();
        NetInstant {
            micros: time_spec.tv_sec as i64 * 1000_000 + time_spec.tv_nsec as i64 / 1000,
        }
    }
    fn yield_now(&self) -> bool {
        shim::suspend();
        // interrupt by signal ?
        let task = shim::current_task().unwrap();
        task.have_signal()
    }
}
