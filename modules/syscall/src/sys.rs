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
