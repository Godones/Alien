use simple_net::common::NetError;
use syscall_define::LinuxErrno;

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
