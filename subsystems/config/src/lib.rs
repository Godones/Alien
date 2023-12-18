//! 配置文件
#![no_std]

/// Alien os的标志
pub const ALIEN_FLAG: &str = r"
     _      _   _
    / \    | | (_)   ___   _ __
   / _ \   | | | |  / _ \ | '_ \
  / ___ \  | | | | |  __/ | | | |
 /_/   \_\ |_| |_|  \___| |_| |_|
";

/// qemu时钟频率
pub const CLOCK_FREQ: usize = 1250_0000;

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

/// (32+6)MB
const HEAP_SIZE: usize = 0x26_00000;
pub const KERNEL_HEAP_SIZE: usize = HEAP_SIZE;

///qemu的设备地址空间
pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC  in virt machine
    (0x2000000, 0x10000),
    (0xc00_0000, 0x21_0000), // VIRT_PLIC in virt machine
    (0x1000_0000, 0x9000),   // VIRT_UART0 with GPU  in virt machine
    (0x3000_0000, 0x1000_0000),
];
