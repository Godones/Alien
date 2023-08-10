use numeric_enum_macro::numeric_enum;

numeric_enum! {
    #[repr(usize)]
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    #[allow(non_camel_case_types)]
    /// Generic musl socket domain.
    pub enum Domain {
        /// Local communication
        AF_UNIX = 1,
        /// IPv4 Internet protocols
        AF_INET = 2,
    }
}
numeric_enum! {
    #[repr(usize)]
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
}
pub const SOCKET_TYPE_MASK: u32 = 0xff;

numeric_enum! {
    #[repr(usize)]
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub enum ShutdownFlag {
        /// 禁用接收
        SHUTRD = 0,
        /// 禁用传输
        SHUTWR = 1,
        /// 同时禁用socket的的传输和接收功能
        SHUTRDWR = 2,
    }
}

pub const LOCAL_LOOPBACK_ADDR: u32 = 0x7f000001;

numeric_enum! {
    #[repr(usize)]
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub enum SocketLevel {
        Ip = 0,
        Socket = 1,
        Tcp = 6,
    }
}

numeric_enum! {
    #[repr(usize)]
    #[allow(non_camel_case_types)]
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub enum SocketOption {
        SOL_REUSEADDR = 2,
        SOL_DONTROUTE = 5,
        SOL_SNDBUF = 7,
        SOL_RCVBUF = 8,
        SOL_KEEPALIVE = 9,
        SOL_RCVTIMEO = 20,
    }
}

numeric_enum! {
    #[repr(usize)]
    #[allow(non_camel_case_types)]
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub enum TcpSocketOption {
        TCP_NODELAY = 1, // disable nagle algorithm and flush
        TCP_MAXSEG = 2,
        TCP_INFO = 11,
        TCP_CONGESTION = 13,
    }
}
