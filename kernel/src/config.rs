pub const FLAG:&str = r"
    __  ___   ____     ____    __  __    __     ______          ____    _____
   /  |/  /  / __ \   / __ \  / / / /   / /    / ____/         / __ \  / ___/
  / /|_/ /  / / / /  / / / / / / / /   / /    / __/    ______ / / / /  \__ \
 / /  / /  / /_/ /  / /_/ / / /_/ /   / /___ / /___   /_____// /_/ /  ___/ /
/_/  /_/   \____/  /_____/  \____/   /_____//_____/          \____/  /____/

";

// 0x8020_0000
pub const RISCV_UART_ADDR: usize = 0x10000000;
pub const RISCV_UART_RANG: usize = 0x100;
// pub const MEMORY_END: usize = 0x1_0000_0000; //2GB
pub const MEMORY_END: usize = 0x8820_0000; //2GB
pub const FRAME_SIZE: usize = 0x1000; //4KB
