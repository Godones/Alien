pub const FLAG: &str = r"
    __  ___   ____     ____    __  __    __     ______          ____    _____
   /  |/  /  / __ \   / __ \  / / / /   / /    / ____/         / __ \  / ___/
  / /|_/ /  / / / /  / / / / / / / /   / /    / __/    ______ / / / /  \__ \
 / /  / /  / /_/ /  / /_/ / / /_/ /   / /___ / /___   /_____// /_/ /  ___/ /
/_/  /_/   \____/  /_____/  \____/   /_____//_____/          \____/  /____/

";

// 0x8020_0000
pub const RISCV_UART_ADDR: usize = 0x10_000_000;
pub const RISCV_UART_RANG: usize = 0x100;
pub const MEMORY_END: usize = 0x88_000_000; //128
                                          // pub const MEMORY_END: usize = 0x8820_0000; //2GB
pub const FRAME_SIZE: usize = 0x1000; //4KB
pub const FRAME_BITS: usize = 12;

pub const TIMER_FREQ: usize = 10_000_000;
pub const CPU_NUM: usize = 1;

pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC  in virt machine
    (0x2000000, 0x10000),
    (0xc000000, 0x210000),  // VIRT_PLIC in virt machine
    (0x10_000_000, 0x9000), // VIRT_UART0 with GPU  in virt machine
];

// 跳板页定义
pub const TRAMPOLINE: usize = usize::MAX - FRAME_SIZE + 1;
pub const TRAP_CONTEXT_BASE: usize = TRAMPOLINE - FRAME_SIZE;

// 栈大小8k
pub const KERNEL_STACK_SIZE: usize = 0x2000;
pub const USER_STACK_SIZE: usize = 0x2000;

// 进程数量/线程数量/描述符表大小限制
pub const MAX_PROCESS_NUM: usize = 1024;
pub const MAX_THREAD_NUM: usize = 1024;
pub const MAX_SUB_PROCESS_NUM: usize = 1024;
pub const MAX_FD_NUM: usize = 64;
