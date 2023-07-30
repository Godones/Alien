pub const FLAG: &str = r"
     _      _   _
    / \    | | (_)   ___   _ __
   / _ \   | | | |  / _ \ | '_ \
  / ___ \  | | | | |  __/ | | | |
 /_/   \_\ |_| |_|  \___| |_| |_|
";
#[cfg(feature = "qemu")]
pub const CLOCK_FREQ: usize = 12500000;
#[cfg(feature = "vf2")]
pub const CLOCK_FREQ: usize = 400_0000;

#[cfg(feature = "hifive")]
pub const CLOCK_FREQ: usize = 100_0000;

#[cfg(feature = "cv1811h")]
pub const CLOCK_FREQ: usize = 0x17d7840;

pub const FRAME_SIZE: usize = 0x1000;
pub const FRAME_BITS: usize = 12;
pub const STACK_SIZE: usize = 1024 * 64;
pub const STACK_SIZE_BITS: usize = 16;

pub const TIMER_FREQ: usize = CLOCK_FREQ;
pub const CPU_NUM: usize = 1;

#[cfg(feature = "qemu")]
pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC  in virt machine
    (0x2000000, 0x10000),
    (0xc00_0000, 0x21_0000), // VIRT_PLIC in virt machine
    (0x1000_0000, 0x9000),   // VIRT_UART0 with GPU  in virt machine
    (0x3000_0000, 0x1000_0000),
];

#[cfg(feature = "vf2")]
pub const MMIO: &[(usize, usize)] = &[
    (0x17040000, 0x10000),     // RTC
    (0xc000000, 0x4000000),    //PLIC
    (0x00_1000_0000, 0x10000), // UART
];

pub const FRAME_MAX_ORDER: usize = 16;

// todo!(if the app linker script changed, this should be changed too)
pub const PROCESS_HEAP_MAX: usize = u32::MAX as usize + 1;
// 跳板页定义
pub const TRAMPOLINE: usize = usize::MAX - 2 * FRAME_SIZE + 1;
pub const TRAP_CONTEXT_BASE: usize = TRAMPOLINE - FRAME_SIZE;

// app内核栈大小
pub const USER_KERNEL_STACK_SIZE: usize = 0x1000 * 2;
// 用户栈大小
pub const USER_STACK_SIZE: usize = 0x50_000;

#[cfg(any(feature = "vf2", feature = "hifive"))]
pub const BLOCK_CACHE_FRAMES: usize = 1024 * 4;
#[cfg(feature = "qemu")]
pub const BLOCK_CACHE_FRAMES: usize = 512;

#[cfg(any(feature = "vf2", feature = "hifive"))]
pub const HEAP_SIZE: usize = 0x40_00000;
#[cfg(feature = "qemu")]
pub const HEAP_SIZE: usize = 0x26_00000; // (32+6)MB

#[cfg(any(feature = "talloc", feature = "buddy"))]
pub const KERNEL_HEAP_SIZE: usize = HEAP_SIZE;

pub const PIPE_BUF: usize = 4096;

// 线程数量/描述符表大小限制
pub const MAX_THREAD_NUM: usize = 65536;
pub const MAX_SUB_PROCESS_NUM: usize = 1024;
pub const MAX_FD_NUM: usize = 4096;

pub const MAX_INPUT_EVENT_NUM: usize = 1024;

pub const MAX_SOCKET_DATA_LEN: usize = 1024 * 4;

/// 如果 elf 的 phdr 指示 base 是 0(如 libc-test 的 libc.so)，则需要找一个非0的位置放置
pub const ELF_BASE_RELOCATE: usize = 0x400_0000;

pub const UTC: &[u8] = &[
    b'T', b'Z', b'i', b'f', b'2', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x1, 0, 0,
    0, 0x1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x1, 0, 0, 0, 0x4, 0, 0, 0, 0, 0, 0, b'U', b'T', b'C',
    0, 0, 0, b'T', b'Z', b'i', b'f', b'2', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0x1, 0, 0, 0, 0x1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x1, 0, 0, 0, 0x4, 0, 0, 0, 0, 0, 0, b'U',
    b'T', b'C', 0, 0, 0, 0x0a, 0x55, 0x54, 0x43, 0x30, 0x0a,
];

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

pub const PASSWORD: &str = r"
root:x:0:0:root:/root:/bin/bash
";
