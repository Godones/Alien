use simple_net::{KernelNetFunc, NetInstant};

use crate::interrupt::DeviceBase;
use crate::task::do_suspend;
use crate::timer::TimeSpec;

pub trait NetDevice: DeviceBase {}

#[derive(Debug, Default)]
pub struct NetNeedFunc;

impl KernelNetFunc for NetNeedFunc {
    fn now(&self) -> NetInstant {
        let time_spec = TimeSpec::now();
        NetInstant {
            micros: time_spec.tv_sec as i64 * 1000_000 + time_spec.tv_nsec as i64 / 1000,
        }
    }
    fn yield_now(&self) {
        do_suspend();
    }
}
