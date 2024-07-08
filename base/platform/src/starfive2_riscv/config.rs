pub const CLOCK_FREQ: usize = 400_0000;

/// vf2的设备地址空间
pub const DEVICE_SPACE: &[(&str, usize, usize)] = &[
    ("rtc", 0x17040000, 0x10000),
    ("plic", 0xc000000, 0x4000000),
    ("virtio-mmio", 0x00_1000_0000, 0x10000),
];
