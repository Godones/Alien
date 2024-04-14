use int_enum::IntEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, IntEnum)]
#[repr(u8)]
pub enum VirtioMmioVersion {
    Legacy = 1,
    Modern = 2,
}

impl TryFrom<u32> for VirtioMmioVersion {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Legacy),
            2 => Ok(Self::Modern),
            _ => Err(()),
        }
    }
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntEnum)]
pub enum VirtioMmioDeviceType {
    Invalid = 0,
    Network = 1,
    Block = 2,
    Console = 3,
    EntropySource = 4,
    MemoryBallooning = 5,
    IoMemory = 6,
    Rpmsg = 7,
    ScsiHost = 8,
    _9P = 9,
    Mac80211 = 10,
    RprocSerial = 11,
    VirtioCAIF = 12,
    MemoryBalloon = 13,
    GPU = 16,
    Timer = 17,
    Input = 18,
    Socket = 19,
    Crypto = 20,
    SignalDistributionModule = 21,
    Pstore = 22,
    IOMMU = 23,
    Memory = 24,
    Audio = 25,
    Filesystem = 26,
    Pmem = 27,
    Rpmb = 28,
    Wireless = 29,
    VideoEncoder = 30,
    VideoDecoder = 31,
    Scmi = 32,
    NitroSecure = 33,
    I2CAdapter = 34,
    Watchdog = 35,
    CAN = 36,
    ParameterServer = 38,
    AudioPolicy = 39,
    Bluetooth = 40,
    GPIO = 41,
    RDMA = 42,
}
