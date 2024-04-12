use int_enum::IntEnum;
use pod::Pod;

use crate::time::TimeVal;

#[repr(u32)]
#[derive(Eq, PartialEq, Debug, Copy, Clone, IntEnum)]
pub enum SyslogAction {
    CLOSE = 0,
    OPEN = 1,
    READ = 2,
    ReadAll = 3,
    ReadClear = 4,
    CLEAR = 5,
    ConsoleOff = 6,
    ConsoleOn = 7,
    ConsoleLevel = 8,
    SizeUnread = 9,
    SizeBuffer = 10,
    Unknown = 11,
}

#[derive(Clone, Copy, Debug, Pod)]
#[repr(C)]
pub struct Sysinfo {
    /// Seconds since boot
    pub uptime: usize,
    /// 1, 5, and 15 minute load averages
    pub loads: [usize; 3],
    /// Total usable main memory size
    pub totalram: usize,
    /// Available memory size
    pub freeram: usize,
    /// Amount of shared memory
    pub sharedram: usize,
    /// Memory used by buffers
    pub bufferram: usize,
    /// Total swap space size
    pub totalswap: usize,
    /// Swap space still available
    pub freeswap: usize,
    /// Number of current processes
    pub procs: u16,
    /// Total high memory size
    pub totalhigh: usize,
    /// Available high memory size
    pub freehigh: usize,
    /// Memory unit size in bytes
    pub mem_unit: u32,
    //char __reserved[256];
    // In the above structure, sizes of the memory and swap fields are given as multiples of mem_unit bytes.
}

#[repr(C)]
#[derive(Clone, Copy, Default, Pod)]
pub struct Rusage {
    /// user CPU time used
    pub ru_utime: TimeVal,
    /// system CPU time used
    pub ru_stime: TimeVal,
    /// (NOT IMPLEMENTED) maximum resident set size
    ru_maxrss: isize,
    /// (NOT IMPLEMENTED) integral shared memory size
    ru_ixrss: isize,
    /// (NOT IMPLEMENTED) integral unshared data size
    ru_idrss: isize,
    /// (NOT IMPLEMENTED) integral unshared stack size
    ru_isrss: isize,
    /// (NOT IMPLEMENTED) page reclaims (soft page faults)
    ru_minflt: isize,
    /// (NOT IMPLEMENTED) page faults (hard page faults)
    ru_majflt: isize,
    /// (NOT IMPLEMENTED) swaps
    ru_nswap: isize,
    /// (NOT IMPLEMENTED) block input operations
    ru_inblock: isize,
    /// (NOT IMPLEMENTED) block output operations
    ru_oublock: isize,
    /// (NOT IMPLEMENTED) IPC messages sent
    ru_msgsnd: isize,
    /// (NOT IMPLEMENTED) IPC messages received
    ru_msgrcv: isize,
    /// (NOT IMPLEMENTED) signals received
    ru_nsignals: isize,
    /// (NOT IMPLEMENTED) voluntary context switches
    ru_nvcsw: isize,
    /// (NOT IMPLEMENTED) involuntary context switches
    ru_nivcsw: isize,
}

#[repr(isize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntEnum)]
pub enum RusageFlag {
    /// Returns the resource usage of the calling process
    RusageSelf = 0,
    /// Returns the resource usage of all children of the calling process that have
    /// terminated and been waited for
    RusageChildren = -1,
    /// Returns the resource usage of the calling thread
    RusageThread = 1,
}
