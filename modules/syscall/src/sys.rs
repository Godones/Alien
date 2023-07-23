use numeric_enum_macro::numeric_enum;

numeric_enum! {
    #[repr(u32)]
    #[derive(Eq, PartialEq, Debug, Copy, Clone)]
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
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Sysinfo {
    pub uptime: usize,
    /* Seconds since boot */
    pub loads: [usize; 3],
    /* 1, 5, and 15 minute load averages */
    pub totalram: usize,
    /* Total usable main memory size */
    pub freeram: usize,
    /* Available memory size */
    pub sharedram: usize,
    /* Amount of shared memory */
    pub bufferram: usize,
    /* Memory used by buffers */
    pub totalswap: usize,
    /* Total swap space size */
    pub freeswap: usize,
    /* Swap space still available */
    pub procs: u16,
    /* Number of current processes */
    pub totalhigh: usize,
    /* Total high memory size */
    pub freehigh: usize,
    /* Available high memory size */
    pub mem_unit: u32,
    /* Memory unit size in bytes */
    //char __reserved[256];
    // In the above structure, sizes of the memory and swap fields are given as multiples of mem_unit bytes.
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Rusage {
    pub ru_utime: TimeVal,
    /* user CPU time used */
    pub ru_stime: TimeVal,
    /* system CPU time used */
    ru_maxrss: isize,
    // NOT IMPLEMENTED /* maximum resident set size */
    ru_ixrss: isize,
    // NOT IMPLEMENTED /* integral shared memory size */
    ru_idrss: isize,
    // NOT IMPLEMENTED /* integral unshared data size */
    ru_isrss: isize,
    // NOT IMPLEMENTED /* integral unshared stack size */
    ru_minflt: isize,
    // NOT IMPLEMENTED /* page reclaims (soft page faults) */
    ru_majflt: isize,
    // NOT IMPLEMENTED /* page faults (hard page faults) */
    ru_nswap: isize,
    // NOT IMPLEMENTED /* swaps */
    ru_inblock: isize,
    // NOT IMPLEMENTED /* block input operations */
    ru_oublock: isize,
    // NOT IMPLEMENTED /* block output operations */
    ru_msgsnd: isize,
    // NOT IMPLEMENTED /* IPC messages sent */
    ru_msgrcv: isize,
    // NOT IMPLEMENTED /* IPC messages received */
    ru_nsignals: isize,
    // NOT IMPLEMENTED /* signals received */
    ru_nvcsw: isize,
    // NOT IMPLEMENTED /* voluntary context switches */
    ru_nivcsw: isize, // NOT IMPLEMENTED /* involuntary context switches */
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct TimeVal {
    /// seconds
    pub tv_sec: usize,
    /// microseconds
    pub tv_usec: usize,
}

impl TimeVal {
    pub fn new() -> Self {
        Self {
            tv_sec: 0,
            tv_usec: 0,
        }
    }
}

const USEC_PER_SEC: usize = 1000_000;

impl From<usize> for TimeVal {
    fn from(value: usize) -> Self {
        Self {
            tv_sec: value / USEC_PER_SEC,
            tv_usec: value % USEC_PER_SEC,
        }
    }
}

impl Into<usize> for TimeVal {
    fn into(self) -> usize {
        self.tv_sec * USEC_PER_SEC + self.tv_usec
    }
}

impl Rusage {
    pub fn new() -> Self {
        Self {
            ru_utime: TimeVal::new(),
            ru_stime: TimeVal::new(),
            ru_maxrss: 0,
            ru_ixrss: 0,
            ru_idrss: 0,
            ru_isrss: 0,
            ru_minflt: 0,
            ru_majflt: 0,
            ru_nswap: 0,
            ru_inblock: 0,
            ru_oublock: 0,
            ru_msgsnd: 0,
            ru_msgrcv: 0,
            ru_nsignals: 0,
            ru_nvcsw: 0,
            ru_nivcsw: 0,
        }
    }
}
