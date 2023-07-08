pub const FLAG: &str = r"
     _      _   _
    / \    | | (_)   ___   _ __
   / _ \   | | | |  / _ \ | '_ \
  / ___ \  | | | | |  __/ | | | |
 /_/   \_\ |_| |_|  \___| |_| |_|
";
#[cfg(not(feature = "vf2"))]
pub const CLOCK_FREQ: usize = 12500000;
#[cfg(feature = "vf2")]
pub const CLOCK_FREQ: usize = 4000000;

#[cfg(feature = "cv1811h")]
pub const CLOCK_FREQ: usize = 0x17d7840;

pub const FRAME_SIZE: usize = 0x1000;
pub const FRAME_BITS: usize = 12;
pub const STACK_SIZE: usize = 1024 * 64;
pub const STACK_SIZE_BITS: usize = 16;

pub const TIMER_FREQ: usize = CLOCK_FREQ;
pub const CPU_NUM: usize = 1;

pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC  in virt machine
    (0x2000000, 0x10000),
    (0xc00_0000, 0x21_0000), // VIRT_PLIC in virt machine
    (0x1000_0000, 0x9000),   // VIRT_UART0 with GPU  in virt machine
    (0x3000_0000, 0x1000_0000),
];

pub const FRAME_MAX_ORDER: usize = 16;

// todo!(if the app linker script changed, this should be changed too)
pub const PROCESS_HEAP_MAX: usize = u32::MAX as usize + 1;
// 2^32 4GB
// 跳板页定义
pub const TRAMPOLINE: usize = usize::MAX - 2 * FRAME_SIZE + 1;
pub const TRAP_CONTEXT_BASE: usize = TRAMPOLINE - FRAME_SIZE;

// 栈大小16k
pub const USER_KERNEL_STACK_SIZE: usize = 0x1000 * 16;
// 64KB
pub const USER_STACK_SIZE: usize = 0x8000;

// 进程数量/线程数量/描述符表大小限制
pub const MAX_PROCESS_NUM: usize = 1024;
pub const MAX_THREAD_NUM: usize = 65536;
pub const MAX_SUB_PROCESS_NUM: usize = 1024;
pub const MAX_FD_NUM: usize = 1024;

pub const MAX_INPUT_EVENT_NUM: usize = 1024;
