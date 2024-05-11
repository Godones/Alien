use core::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use int_enum::IntEnum;
use pod::Pod;

#[repr(usize)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, IntEnum)]
#[allow(non_camel_case_types)]
/// Generic musl socket domain.
pub enum Domain {
    /// Local communication
    AF_UNIX = 1,
    /// IPv4 Internet protocols
    AF_INET = 2,
}

#[repr(usize)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, IntEnum)]
#[allow(non_camel_case_types)]
/// Generic musl socket type.
pub enum SocketType {
    /// Provides sequenced, reliable, two-way, connection-based byte streams.
    /// An out-of-band data transmission mechanism may be supported.
    SOCK_STREAM = 1,
    /// Supports datagrams (connectionless, unreliable messages of a fixed maximum length).
    SOCK_DGRAM = 2,
    /// Provides raw network protocol access.
    SOCK_RAW = 3,
    /// Provides a reliable datagram layer that does not guarantee ordering.
    SOCK_RDM = 4,
    /// Provides a sequenced, reliable, two-way connection-based data
    /// transmission path for datagrams of fixed maximum length;
    /// a consumer is required to read an entire packet with each input system call.
    SOCK_SEQPACKET = 5,
    /// Datagram Congestion Control Protocol socket
    SOCK_DCCP = 6,
    /// Obsolete and should not be used in new programs.
    SOCK_PACKET = 10,
    /// Set O_NONBLOCK flag on the open fd
    SOCK_NONBLOCK = 0x800,
    /// Set FD_CLOEXEC flag on the new fd
    SOCK_CLOEXEC = 0x80000,
}

pub const SOCKET_TYPE_MASK: u32 = 0xff;

#[repr(usize)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, IntEnum)]
pub enum ShutdownFlag {
    /// 禁用接收
    SHUTRD = 0,
    /// 禁用传输
    SHUTWR = 1,
    /// 同时禁用socket的的传输和接收功能
    SHUTRDWR = 2,
}
pub const LOCAL_LOOPBACK_ADDR: u32 = 0x7f000001;

#[repr(usize)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, IntEnum)]
pub enum SocketLevel {
    Ip = 0,
    Socket = 1,
    Tcp = 6,
}

#[repr(usize)]
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, IntEnum)]
pub enum SocketOption {
    SO_DEBUG = 1,
    SO_REUSEADDR = 2,
    SO_TYPE = 3,
    SO_ERROR = 4,
    SO_DONTROUTE = 5,
    SO_BROADCAST = 6,
    SO_SNDBUF = 7,
    SO_RCVBUF = 8,
    SO_SNDBUFFORCE = 32,
    SO_RCVBUFFORCE = 33,
    SO_KEEPALIVE = 9,
    SO_OOBINLINE = 10,
    SO_NO_CHECK = 11,
    SO_PRIORITY = 12,
    SO_LINGER = 13,
    SO_BSDCOMPAT = 14,
    SO_REUSEPORT = 15,
    SO_PASSCRED = 16,
    SO_PEERCRED = 17,
    SO_RCVLOWAT = 18,
    SO_SNDLOWAT = 19,
    SO_RCVTIMEO_OLD = 20,
    SO_SNDTIMEO_OLD = 21,
}
#[repr(usize)]
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, IntEnum)]
pub enum TcpSocketOption {
    TCP_NODELAY = 1, // disable nagle algorithm and flush
    TCP_MAXSEG = 2,
    TCP_INFO = 11,
    TCP_CONGESTION = 13,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod)]
pub struct SocketAddrInRaw {
    pub family: u16,
    pub in_port: u16,
    pub addr: [u8; 4],
    pub sin_zero: [u8; 8],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SocketAddrIn {
    pub family: u16,
    pub in_port: u16,
    pub addr: Ipv4Addr,
    pub sin_zero: [u8; 8],
}

impl From<SocketAddrIn> for SocketAddr {
    fn from(value: SocketAddrIn) -> Self {
        SocketAddr::V4(SocketAddrV4::new(value.addr, value.in_port))
    }
}

impl Default for SocketAddrIn {
    fn default() -> Self {
        Self {
            family: Domain::AF_INET as u16,
            in_port: 0,
            addr: Ipv4Addr::new(0, 0, 0, 0),
            sin_zero: [0; 8],
        }
    }
}

impl From<SocketAddrInRaw> for SocketAddrIn {
    fn from(value: SocketAddrInRaw) -> Self {
        Self {
            family: value.family,
            in_port: value.in_port,
            addr: Ipv4Addr::from(value.addr),
            sin_zero: value.sin_zero,
        }
    }
}

impl From<SocketAddrIn> for SocketAddrInRaw {
    fn from(value: SocketAddrIn) -> Self {
        Self {
            family: value.family,
            in_port: value.in_port,
            addr: value.addr.octets(),
            sin_zero: value.sin_zero,
        }
    }
}
