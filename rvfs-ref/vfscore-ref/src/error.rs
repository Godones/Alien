use core::{
    error::Error,
    fmt::{Debug, Display, Formatter},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VfsError {
    /// EACCES 权限拒绝
    PermissionDenied = 1,
    /// ENOENT 无此文件或目录
    NoEntry = 2,
    /// 被打断
    EINTR = 4,
    /// EIO 输入输出错误
    IoError = 5,
    /// try again
    EAGAIN = 11,
    /// ENOMEM 内存不足
    NoMem = 12,
    /// EACCESS 无访问权限
    Access = 13,
    /// EBUSY 设备或资源忙
    EBUSY = 16,
    /// EEXIST 文件已存在
    EExist = 17,
    /// ENOTDIR 不是目录
    NotDir = 20,
    /// EINVAL 无效参数
    Invalid = 22,
    /// ENODEV 设备不存在
    NoDev = 19,
    /// IsDir 是目录
    IsDir = 21,
    /// ENOTTY 不是终端
    NoTTY = 25,
    /// ENOSPC 空间不足
    NoSpace = 28,
    /// Illegal seek
    ESPIPE = 29,
    /// Broken pipe
    EPIPE = 32,
    /// ENAMETOOLONG 名称太长
    NameTooLong = 36,
    /// ENOSYS 不支持的系统调用
    NoSys = 38,
    /// ENOTEMPTY  目录非空
    NotEmpty = 39,
}

impl Display for VfsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            VfsError::EAGAIN => {
                write!(f, "Try again")
            }
            VfsError::EINTR => {
                write!(f, "Interrupted system call")
            }
            VfsError::EPIPE => {
                write!(f, "Broken pipe")
            }
            VfsError::ESPIPE => {
                write!(f, "Illegal seek")
            }
            VfsError::PermissionDenied => {
                write!(f, "Permission denied")
            }
            VfsError::NoEntry => {
                write!(f, "No such file or directory")
            }
            VfsError::IoError => {
                write!(f, "Input/output error")
            }
            VfsError::EExist => {
                write!(f, "File exists")
            }
            VfsError::NotDir => {
                write!(f, "Not a directory")
            }
            VfsError::NotEmpty => {
                write!(f, "Directory not empty")
            }
            VfsError::NoMem => {
                write!(f, "Out of memory")
            }
            VfsError::NoSpace => {
                write!(f, "No space left on device")
            }
            VfsError::Invalid => {
                write!(f, "Invalid argument")
            }
            VfsError::NameTooLong => {
                write!(f, "File name too long")
            }
            VfsError::NoSys => {
                write!(f, "Function not implemented")
            }
            VfsError::NoDev => {
                write!(f, "No such device")
            }
            VfsError::NoTTY => {
                write!(f, "Inappropriate ioctl for device")
            }
            VfsError::IsDir => {
                write!(f, "Is a directory")
            }
            VfsError::Access => {
                write!(f, "Access error")
            }
            VfsError::EBUSY => {
                write!(f, "Device or resource busy")
            }
        }
    }
}

impl Error for VfsError {}

impl From<VfsError> for i32 {
    fn from(value: VfsError) -> Self {
        value as i32
    }
}

impl From<i32> for VfsError {
    fn from(value: i32) -> Self {
        match value {
            1 => VfsError::PermissionDenied,
            2 => VfsError::NoEntry,
            4 => VfsError::EINTR,
            5 => VfsError::IoError,
            11 => VfsError::EAGAIN,
            12 => VfsError::NoMem,
            13 => VfsError::Access,
            16 => VfsError::EBUSY,
            17 => VfsError::EExist,
            20 => VfsError::NotDir,
            22 => VfsError::Invalid,
            19 => VfsError::NoDev,
            21 => VfsError::IsDir,
            25 => VfsError::NoTTY,
            28 => VfsError::NoSpace,
            29 => VfsError::ESPIPE,
            32 => VfsError::EPIPE,
            36 => VfsError::NameTooLong,
            38 => VfsError::NoSys,
            39 => VfsError::NotEmpty,
            _ => VfsError::Invalid,
        }
    }
}

#[cfg(feature = "linux_error")]
impl From<VfsError> for pconst::LinuxErrno {
    fn from(value: VfsError) -> Self {
        pconst::LinuxErrno::try_from(-(i32::from(value) as isize))
            .unwrap_or(pconst::LinuxErrno::EINVAL)
    }
}
#[cfg(feature = "linux_error")]
impl From<pconst::LinuxErrno> for VfsError {
    fn from(value: pconst::LinuxErrno) -> Self {
        let code = -(value as i32);
        VfsError::from(code)
    }
}

#[cfg(test)]
mod tests {
    use super::VfsError;
    #[test]
    fn test_vfs_error() {
        assert_eq!(VfsError::NoEntry as i32, 2);
    }
}
