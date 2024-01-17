pub const CLOCK_FREQ: usize = 1250_0000;
pub const BLOCK_CACHE_FRAMES: usize = 1024 * 4 * 4;
pub const HEAP_SIZE: usize = 0x26_00000; // (32+6)MB

/// qemu的设备地址空间
pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC  in virt machine
    (0x2000000, 0x10000),
    (0xc00_0000, 0x21_0000), // VIRT_PLIC in virt machine
    (0x1000_0000, 0x9000),   // VIRT_UART0 with GPU  in virt machine
    (0x3000_0000, 0x1000_0000),
];
