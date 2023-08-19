//! 现为将网络异常类型 [`NetError`] 转为 系统异常类型 [`LinuxErrno`]的模块。原为定义端口全局变量和操作的模块。

use simple_net::common::NetError;
use syscall_define::LinuxErrno;

/// 现为将网络异常类型 [`NetError`] 转为 系统异常类型 [`LinuxErrno`]。
pub fn neterror2linux(error: NetError) -> LinuxErrno {
    match error {
        NetError::AddrInUse => LinuxErrno::EADDRINUSE,
        NetError::InvalidInput => LinuxErrno::EINVAL,
        NetError::WouldBlock => LinuxErrno::EAGAIN,
        NetError::NotConnected => LinuxErrno::ENOTCONN,
        NetError::BadState => LinuxErrno::EBADF,
        NetError::Unaddressable => LinuxErrno::EADDRNOTAVAIL,
        NetError::AlreadyExists => LinuxErrno::EEXIST,
        NetError::ConnectionRefused => LinuxErrno::ECONNREFUSED,
        NetError::ConnectionReset => LinuxErrno::ECONNRESET,
        NetError::Interrupted => LinuxErrno::EINTR,
    }
}
