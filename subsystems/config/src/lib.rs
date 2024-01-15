#![no_std]
//! 配置文件

/// Alien os的标志
pub const FLAG: &str = r"
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

// QEMU user networking default IP
pub const QEMU_IP: &str = "10.0.2.15";
// QEMU user networking gateway
pub const QEMU_GATEWAY: &str = "10.0.2.2";
