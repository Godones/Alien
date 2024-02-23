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

/// 物理页大小
pub const FRAME_SIZE: usize = 0x1000;
/// 物理页大小的位数
pub const FRAME_BITS: usize = 12;
/// 内核启动栈大小
pub const STACK_SIZE: usize = 1024 * 64;
/// 内核启动栈大小的位数
pub const STACK_SIZE_BITS: usize = 16;

/// 可配置的启动cpu数量
pub const CPU_NUM: usize = 1;

const HEAP_SIZE: usize = 0x26_00000;
pub const KERNEL_HEAP_SIZE: usize = HEAP_SIZE;
