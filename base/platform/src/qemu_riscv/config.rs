pub const CLOCK_FREQ: usize = 1250_0000;
/// qemu的设备地址空间
pub const DEVICE_SPACE: &[(&str, usize, usize)] = &[
    ("test", 0x10_0000, 0x1000),
    ("rtc", 0x10_1000, 0x1000),
    ("clint", 0x200_0000, 0x10000),
    ("plic", 0xc00_0000, 0x600000),
    ("virtio-mmio", 0x1000_0000, 0x9000),
];
