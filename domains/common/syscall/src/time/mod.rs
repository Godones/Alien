use alloc::sync::Arc;

use basic::{
    constants::time::{ClockId, TimeSpec},
    AlienError, AlienResult,
};
use interface::TaskDomain;
use pod::Pod;

pub fn sys_clock_gettime(
    task_domain: &Arc<dyn TaskDomain>,
    clk_id: usize,
    tp: usize,
) -> AlienResult<isize> {
    let id = ClockId::try_from(clk_id).map_err(|_| AlienError::EINVAL)?;
    match id {
        ClockId::Monotonic | ClockId::Realtime | ClockId::ProcessCputimeId => {
            let time_ms = basic::time::read_time_ms();
            let time = TimeSpec {
                tv_sec: (time_ms / 1000) as usize,
                tv_nsec: ((time_ms % 1000) * 1_000_000) as usize,
            };
            task_domain.copy_to_user(tp, time.as_bytes())?;
            Ok(0)
        }
        _ => {
            panic!("clock_get_time: clock_id {:?} not supported", id);
        }
    }
}
