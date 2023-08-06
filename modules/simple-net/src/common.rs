// QEMU user networking default IP
pub const QEMU_IP: &str = "10.0.2.15";
// QEMU user networking gateway
pub const QEMU_GATEWAY: &str = "10.0.2.2";
pub const TCP_RX_BUF_LEN: usize = 64 * 1024;
pub const TCP_TX_BUF_LEN: usize = 64 * 1024;
pub const UDP_RX_BUF_LEN: usize = 64 * 1024;
pub const UDP_TX_BUF_LEN: usize = 64 * 1024;
pub const LISTEN_QUEUE_SIZE: usize = 512;
pub const STANDARD_MTU: usize = 1500;

pub type NetResult<T> = Result<T, NetError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetError {
    AddrInUse,
    InvalidInput,
    WouldBlock,
    NotConnected,
    BadState,
    Unaddressable,
    AlreadyExists,
    ConnectionRefused,
    ConnectionReset,
}

/// Struct for poll result.
#[derive(Debug, Default)]
pub struct NetPollState {
    /// Object can be read now.
    pub readable: bool,
    /// Object can be writen now.
    pub writable: bool,
}
