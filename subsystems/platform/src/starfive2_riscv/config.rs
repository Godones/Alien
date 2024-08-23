pub const CLOCK_FREQ: usize = 400_0000;
pub const BLOCK_CACHE_FRAMES: usize = 1024 * 4 * 4;
pub const HEAP_SIZE: usize = 0x70_00000;

/// vf2的设备地址空间

pub const MMIO: &[(usize, usize)] = &[
    (0x17040000, 0x10000),  // RTC
    (0xc000000, 0x4000000), //PLIC
    (0x1000_0000, 0x10000), // UART
    (0x16020000, 0x10000),  // sdio1
];
