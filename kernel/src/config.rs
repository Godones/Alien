//! 配置文件

/// Alien os的标志
pub const FLAG: &str = r"
     _      _   _
    / \    | | (_)   ___   _ __
   / _ \   | | | |  / _ \ | '_ \
  / ___ \  | | | | |  __/ | | | |
 /_/   \_\ |_| |_|  \___| |_| |_|
";

/// qemu时钟频率
#[cfg(feature = "qemu")]
pub const CLOCK_FREQ: usize = 1250_0000;
/// vf2时钟频率
#[cfg(feature = "vf2")]
pub const CLOCK_FREQ: usize = 400_0000;

/// unmatched时钟频率
#[cfg(feature = "hifive")]
pub const CLOCK_FREQ: usize = 100_0000;

/// cv1811h时钟频率
#[cfg(feature = "cv1811h")]
pub const CLOCK_FREQ: usize = 0x17d7840;

/// 物理页大小
pub const FRAME_SIZE: usize = 0x1000;
/// 物理页大小的位数
pub const FRAME_BITS: usize = 12;
/// 内核启动栈大小
pub const STACK_SIZE: usize = 1024 * 64;
/// 内核启动栈大小的位数
pub const STACK_SIZE_BITS: usize = 16;

/// equal to CLOCK_FREQ
pub const TIMER_FREQ: usize = CLOCK_FREQ;
/// 可配置的启动cpu数量
pub const CPU_NUM: usize = 1;

pub const AT_FDCWD: isize = -100isize;

///qemu的设备地址空间
#[cfg(feature = "qemu")]
pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC  in virt machine
    (0x2000000, 0x10000),
    (0xc00_0000, 0x21_0000), // VIRT_PLIC in virt machine
    (0x1000_0000, 0x9000),   // VIRT_UART0 with GPU  in virt machine
    (0x3000_0000, 0x1000_0000),
];

/// vf2的设备地址空间
#[cfg(feature = "vf2")]
pub const MMIO: &[(usize, usize)] = &[
    (0x17040000, 0x10000),     // RTC
    (0xc000000, 0x4000000),    //PLIC
    (0x00_1000_0000, 0x10000), // UART
];

/// hifive的设备地址空间
#[cfg(feature = "hifive")]
pub const MMIO: &[(usize, usize)] = &[
    (0xc000000, 0x4000000), //PLIC
];

// todo!(if the app linker script changed, this should be changed too)
/// 进程的堆空间上限
pub const PROCESS_HEAP_MAX: usize = u32::MAX as usize + 1;
/// 跳板页的虚拟地址
pub const TRAMPOLINE: usize = usize::MAX - 2 * FRAME_SIZE + 1;
/// trap context的虚拟地址
pub const TRAP_CONTEXT_BASE: usize = TRAMPOLINE - FRAME_SIZE;

/// app内核栈大小
pub const USER_KERNEL_STACK_SIZE: usize = 0x1000 * 2;
/// app用户栈大小
pub const USER_STACK_SIZE: usize = 0x50_000;

/// vf2/unmatched 的块缓存大小
#[cfg(any(feature = "vf2", feature = "hifive"))]
pub const BLOCK_CACHE_FRAMES: usize = 1024 * 4 * 4;

/// qemu 的块缓存大小
#[cfg(feature = "qemu")]
pub const BLOCK_CACHE_FRAMES: usize = 1024 * 4 * 4;

/// vf2/unmatched 的堆空间大小
#[cfg(any(feature = "vf2", feature = "hifive"))]
pub const HEAP_SIZE: usize = 0x40_00000;

/// qemu 的堆空间大小
#[cfg(feature = "qemu")]
pub const HEAP_SIZE: usize = 0x26_00000; // (32+6)MB

/// equal to HEAP_SIZe
#[cfg(any(feature = "talloc", feature = "buddy"))]
pub const KERNEL_HEAP_SIZE: usize = HEAP_SIZE;

/// pipe缓冲区大小
pub const PIPE_BUF: usize = 65536;

/// 线程数量大小限制
pub const MAX_THREAD_NUM: usize = 65536;
/// 描述符数量大小限制
pub const MAX_FD_NUM: usize = 4096;

/// 最大的输入事件数量
pub const MAX_INPUT_EVENT_NUM: usize = 1024;

/// 如果 elf 的 phdr 指示 base 是 0(如 libc-test 的 libc.so)，则需要找一个非0的位置放置
/// 我们将其从 0x4000_0000 开始放置。主要用于动态链接库使用
pub const ELF_BASE_RELOCATE: usize = 0x400_0000;

/// localtime文件中保存的内容
pub const UTC: &[u8] = &[
    b'T', b'Z', b'i', b'f', b'2', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x1, 0, 0,
    0, 0x1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x1, 0, 0, 0, 0x4, 0, 0, 0, 0, 0, 0, b'U', b'T', b'C',
    0, 0, 0, b'T', b'Z', b'i', b'f', b'2', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0x1, 0, 0, 0, 0x1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x1, 0, 0, 0, 0x4, 0, 0, 0, 0, 0, 0, b'U',
    b'T', b'C', 0, 0, 0, 0x0a, 0x55, 0x54, 0x43, 0x30, 0x0a,
];

/// rtc文件中保存的内容
pub const RTC_TIME: &str = r"
rtc_time	: 03:01:50
rtc_date	: 2023-07-11
alrm_time	: 13:03:24
alrm_date	: 2023-07-11
alarm_IRQ	: no
alrm_pending	: no
update IRQ enabled	: no
periodic IRQ enabled	: no
periodic IRQ frequency	: 1024
max user IRQ frequency	: 64
24hr		: yes
periodic_IRQ	: no
update_IRQ	: no
HPET_emulated	: no
BCD		: yes
DST_enable	: no
periodic_freq	: 1024
batt_status	: okay";

/// meminfo文件中保存的内容
pub const MEMINFO: &str = r"
MemTotal:         944564 kB
MemFree:          835248 kB
MemAvailable:     873464 kB
Buffers:            6848 kB
Cached:            36684 kB
SwapCached:            0 kB
Active:            19032 kB
Inactive:          32676 kB
Active(anon):        128 kB
Inactive(anon):     8260 kB
Active(file):      18904 kB
Inactive(file):    24416 kB
Unevictable:           0 kB
Mlocked:               0 kB
SwapTotal:             0 kB
SwapFree:              0 kB
Dirty:                 0 kB
Writeback:             0 kB
AnonPages:          8172 kB
Mapped:            16376 kB
Shmem:               216 kB
KReclaimable:       9960 kB
Slab:              17868 kB
SReclaimable:       9960 kB
SUnreclaim:         7908 kB
KernelStack:        1072 kB
PageTables:          600 kB
NFS_Unstable:          0 kB
Bounce:                0 kB
WritebackTmp:          0 kB
CommitLimit:      472280 kB
Committed_AS:      64684 kB
VmallocTotal:   67108863 kB
VmallocUsed:       15740 kB
VmallocChunk:          0 kB
Percpu:              496 kB
HugePages_Total:       0
HugePages_Free:        0
HugePages_Rsvd:        0
HugePages_Surp:        0
Hugepagesize:       2048 kB
Hugetlb:               0 kB
";

// QEMU user networking default IP
pub const QEMU_IP: &str = "10.0.2.15";
// QEMU user networking gateway
pub const QEMU_GATEWAY: &str = "10.0.2.2";
