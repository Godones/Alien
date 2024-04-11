// /* Identifier for system-wide realtime clock.  */
// # define CLOCK_REALTIME			0
// /* Monotonic system-wide clock.  */
// # define CLOCK_MONOTONIC		1
// /* High-resolution timer from the CPU.  */
// # define CLOCK_PROCESS_CPUTIME_ID	2
// /* Thread-specific CPU-time clock.  */
// # define CLOCK_THREAD_CPUTIME_ID	3
// /* Monotonic system-wide clock, not adjusted for frequency scaling.  */
// # define CLOCK_MONOTONIC_RAW		4
// /* Identifier for system-wide realtime clock, updated only on ticks.  */
// # define CLOCK_REALTIME_COARSE		5
// /* Monotonic system-wide clock, updated only on ticks.  */
// # define CLOCK_MONOTONIC_COARSE		6
// /* Monotonic system-wide clock that includes time spent in suspension.  */
// # define CLOCK_BOOTTIME			7
// /* Like CLOCK_REALTIME but also wakes suspended system.  */
// # define CLOCK_REALTIME_ALARM		8
// /* Like CLOCK_BOOTTIME but also wakes suspended system.  */
// # define CLOCK_BOOTTIME_ALARM		9
// /* Like CLOCK_REALTIME but in International Atomic Time.  */
// # define CLOCK_TAI			11

use int_enum::IntEnum;

const CLOCK_REALTIME: usize = 0;
const CLOCK_MONOTONIC: usize = 1;
const CLOCK_PROCESS_CPUTIME_ID: usize = 2;
const CLOCK_THREAD_CPUTIME_ID: usize = 3;
const CLOCK_MONOTONIC_RAW: usize = 4;
const CLOCK_REALTIME_COARSE: usize = 5;
const CLOCK_MONOTONIC_COARSE: usize = 6;
const CLOCK_BOOTTIME: usize = 7;
const CLOCK_REALTIME_ALARM: usize = 8;
const CLOCK_BOOTTIME_ALARM: usize = 9;
const CLOCK_TAI: usize = 11;

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntEnum)]
pub enum ClockId {
    Realtime = CLOCK_REALTIME,
    Monotonic = CLOCK_MONOTONIC,
    ProcessCputimeId = CLOCK_PROCESS_CPUTIME_ID,
    ThreadCputimeId = CLOCK_THREAD_CPUTIME_ID,
    MonotonicRaw = CLOCK_MONOTONIC_RAW,
    RealtimeCoarse = CLOCK_REALTIME_COARSE,
    MonotonicCoarse = CLOCK_MONOTONIC_COARSE,
    Boottime = CLOCK_BOOTTIME,
    RealtimeAlarm = CLOCK_REALTIME_ALARM,
    BoottimeAlarm = CLOCK_BOOTTIME_ALARM,
    Tai = CLOCK_TAI,
}

impl ClockId {
    pub fn from_raw(id: usize) -> Option<Self> {
        match id {
            CLOCK_REALTIME => Some(Self::Realtime),
            CLOCK_MONOTONIC => Some(Self::Monotonic),
            CLOCK_PROCESS_CPUTIME_ID => Some(Self::ProcessCputimeId),
            CLOCK_THREAD_CPUTIME_ID => Some(Self::ThreadCputimeId),
            CLOCK_MONOTONIC_RAW => Some(Self::MonotonicRaw),
            CLOCK_REALTIME_COARSE => Some(Self::RealtimeCoarse),
            CLOCK_MONOTONIC_COARSE => Some(Self::MonotonicCoarse),
            CLOCK_BOOTTIME => Some(Self::Boottime),
            CLOCK_REALTIME_ALARM => Some(Self::RealtimeAlarm),
            CLOCK_BOOTTIME_ALARM => Some(Self::BoottimeAlarm),
            CLOCK_TAI => Some(Self::Tai),
            _ => None,
        }
    }
}

#[repr(usize)]
#[allow(non_camel_case_types)]
#[derive(Eq, PartialEq, Debug, Copy, Clone, IntEnum)]
/// sys_settimer / sys_gettimer 中设定的 which，即计时器类型
pub enum TimerType {
    /// 表示目前没有任何计时器(不在linux规范中，是os自己规定的)
    NONE = 999,
    /// 统计系统实际运行时间
    REAL = 0,
    /// 统计用户态运行时间
    VIRTUAL = 1,
    /// 统计进程的所有用户态/内核态运行时间
    PROF = 2,
}
