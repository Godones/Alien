#![no_std]

pub use pconst::*;
pub type AlienError = LinuxErrno;
pub type AlienResult<T> = Result<T, AlienError>;

#[derive(Copy, Clone, Debug, Eq, PartialOrd, PartialEq, Hash, Ord)]
pub struct DeviceId {
    major: u32,
    minor: u32,
}

impl DeviceId {
    pub fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }
    pub fn major(&self) -> u32 {
        self.major
    }
    pub fn minor(&self) -> u32 {
        self.minor
    }
    pub fn id(&self) -> u64 {
        ((self.major as u64) << 32) | (self.minor as u64)
    }
}

impl From<u64> for DeviceId {
    fn from(id: u64) -> Self {
        Self {
            major: (id >> 32) as u32,
            minor: (id & 0xffffffff) as u32,
        }
    }
}

pub const AT_FDCWD: isize = -100isize;
