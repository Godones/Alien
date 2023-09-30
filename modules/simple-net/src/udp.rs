use core::net::SocketAddr;
use core::sync::atomic::{AtomicBool, Ordering};

use log::{debug, error, warn};
use smoltcp::iface::SocketHandle;
use smoltcp::socket::udp::{self, BindError, SendError};
use smoltcp::wire::{IpEndpoint, IpListenEndpoint};
use spin::RwLock;

use crate::common::{NetError, NetPollState, NetResult};
use crate::KERNEL_NET_FUNC;
use kernel_sync::TicketMutex as Mutex;

use super::addr::{from_core_sockaddr, into_core_sockaddr, is_unspecified, UNSPECIFIED_ENDPOINT};
use super::{SocketSetWrapper, SOCKET_SET};

/// A UDP socket that provides POSIX-like APIs.
pub struct UdpSocket {
    handle: SocketHandle,
    local_addr: RwLock<Option<IpEndpoint>>,
    peer_addr: RwLock<Option<IpEndpoint>>,
    nonblock: AtomicBool,
}

impl UdpSocket {
    /// Creates a new UDP socket.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let socket = SocketSetWrapper::new_udp_socket();
        let handle = SOCKET_SET.add(socket);
        Self {
            handle,
            local_addr: RwLock::new(None),
            peer_addr: RwLock::new(None),
            nonblock: AtomicBool::new(false),
        }
    }

    pub fn reuse(&self, handle: SocketHandle) -> Self {
        Self {
            handle,
            local_addr: RwLock::new(self.local_addr.read().clone()),
            peer_addr: RwLock::new(self.peer_addr.read().clone()),
            nonblock: AtomicBool::new(self.nonblock.load(Ordering::Acquire)),
        }
    }

    /// Returns the local address and port, or
    /// [`Err(NotConnected)`](NetError::NotConnected) if not connected.
    pub fn local_addr(&self) -> NetResult<SocketAddr> {
        match self.local_addr.try_read() {
            Some(addr) => addr.map(into_core_sockaddr).ok_or(NetError::NotConnected),
            None => Err(NetError::NotConnected),
        }
    }

    /// Returns the remote address and port, or
    /// [`Err(NotConnected)`](NetError::NotConnected) if not connected.
    pub fn peer_addr(&self) -> NetResult<SocketAddr> {
        self.remote_endpoint().map(into_core_sockaddr)
    }

    /// Returns whether this socket is in nonblocking mode.
    #[inline]
    pub fn is_nonblocking(&self) -> bool {
        self.nonblock.load(Ordering::Acquire)
    }

    /// Moves this UDP socket into or out of nonblocking mode.
    ///
    /// This will result in `recv`, `recv_from`, `send`, and `send_to`
    /// operations becoming nonblocking, i.e., immediately returning from their
    /// calls. If the IO operation is successful, `Ok` is returned and no
    /// further action is required. If the IO operation could not be completed
    /// and needs to be retried, an error with kind
    /// [`Err(WouldBlock)`](NetError::WouldBlock) is returned.
    #[inline]
    pub fn set_nonblocking(&self, nonblocking: bool) {
        self.nonblock.store(nonblocking, Ordering::Release);
    }

    /// Binds an unbound socket to the given address and port.
    ///
    /// It's must be called before [`send_to`](Self::send_to) and
    /// [`recv_from`](Self::recv_from).
    pub fn bind(&self, mut local_addr: SocketAddr) -> NetResult<Option<UdpSocket>> {
        let mut self_local_addr = self.local_addr.write();

        if local_addr.port() == 0 {
            local_addr.set_port(get_ephemeral_port()?);
        }
        if self_local_addr.is_some() {
            warn!("UDP socket {}: already bound", self.handle);
            return Err(NetError::InvalidInput);
        }

        let local_endpoint = from_core_sockaddr(local_addr);
        let endpoint = IpListenEndpoint {
            addr: (!is_unspecified(local_endpoint.addr)).then_some(local_endpoint.addr),
            port: local_endpoint.port,
        };

        *self_local_addr = Some(local_endpoint);
        drop(self_local_addr);

        // let mut udp_reuse = UDP_PORT_REUSE.lock();
        // // check if port is in reuse queue
        // if udp_reuse.contains_key(&local_addr.port()) {
        //     warn!("UDP socket {}: port {} already in use", self.handle, local_addr.port());
        //     // reset self handle to reuse handle
        //     let reuse_handle = udp_reuse.get(&local_addr.port()).unwrap();
        //     let reuse = self.reuse(*reuse_handle);
        //     // it share a  socket handle with reuse socket
        //     return Ok(Some(reuse));
        // }

        SOCKET_SET.with_socket_mut::<udp::Socket, _, _>(self.handle, |socket| {
            socket.bind(endpoint).or_else(|e| match e {
                BindError::InvalidState => {
                    warn!("UDP socket {}: already bound", self.handle);
                    Err(NetError::AlreadyExists)
                }
                BindError::Unaddressable => {
                    warn!("UDP socket {}: invalid address", self.handle);
                    Err(NetError::InvalidInput)
                }
            })
        })?;

        // insert to reuse queue
        // udp_reuse.insert(local_endpoint.port, self.handle);

        debug!("UDP socket {}: bound on {}", self.handle, endpoint);
        Ok(None)
    }

    /// Sends data on the socket to the given address. On success, returns the
    /// number of bytes written.
    pub fn send_to(&self, buf: &[u8], remote_addr: SocketAddr) -> NetResult<usize> {
        if remote_addr.port() == 0 || remote_addr.ip().is_unspecified() {
            warn!("socket send_to() failed: invalid address");
            return Err(NetError::InvalidInput);
        }
        self.send_impl(buf, from_core_sockaddr(remote_addr))
    }

    /// Receives a single datagram message on the socket. On success, returns
    /// the number of bytes read and the origin.
    pub fn recv_from(&self, buf: &mut [u8]) -> NetResult<(usize, SocketAddr)> {
        self.recv_impl(|socket| match socket.recv_slice(buf) {
            Ok((len, meta)) => Ok((len, into_core_sockaddr(meta.endpoint))),
            Err(e) => {
                error!("UDP socket {}: recv_from() failed", self.handle);
                // Err(NetError::BadState)
                Err(NetError::WouldBlock)
            }
        })
    }

    /// Receives a single datagram message on the socket, without removing it from
    /// the queue. On success, returns the number of bytes read and the origin.
    pub fn peek_from(&self, buf: &mut [u8]) -> NetResult<(usize, SocketAddr)> {
        self.recv_impl(|socket| match socket.peek_slice(buf) {
            Ok((len, meta)) => Ok((len, into_core_sockaddr(meta.endpoint))),
            Err(_) => {
                warn!("UDP socket {}: recv_from() failed", self.handle);
                Err(NetError::BadState)
            }
        })
    }

    /// Connects this UDP socket to a remote address, allowing the `send` and
    /// `recv` to be used to send data and also applies filters to only receive
    /// data from the specified address.
    ///
    /// The local port will be generated automatically if the socket is not bound.
    /// It's must be called before [`send`](Self::send) and
    /// [`recv`](Self::recv).
    pub fn connect(&self, addr: SocketAddr) -> NetResult<()> {
        let mut self_peer_addr = self.peer_addr.write();

        if self.local_addr.read().is_none() {
            self.bind(into_core_sockaddr(UNSPECIFIED_ENDPOINT))?;
        }

        *self_peer_addr = Some(from_core_sockaddr(addr));
        debug!("UDP socket {}: connected to {}", self.handle, addr);
        Ok(())
    }

    /// Sends data on the socket to the remote address to which it is connected.
    pub fn send(&self, buf: &[u8]) -> NetResult<usize> {
        let remote_endpoint = self.remote_endpoint()?;
        self.send_impl(buf, remote_endpoint)
    }

    /// Receives a single datagram message on the socket from the remote address
    /// to which it is connected. On success, returns the number of bytes read.
    pub fn recv(&self, buf: &mut [u8]) -> NetResult<usize> {
        let remote_endpoint = self.remote_endpoint()?;
        self.recv_impl(|socket| {
            let (len, meta) = socket.recv_slice(buf).map_err(|_| {
                warn!("UDP socket {}: recv_from() failed", self.handle);
                NetError::BadState
            })?;
            if !is_unspecified(remote_endpoint.addr) && remote_endpoint.addr != meta.endpoint.addr {
                return Err(NetError::WouldBlock);
            }
            if remote_endpoint.port != 0 && remote_endpoint.port != meta.endpoint.port {
                return Err(NetError::WouldBlock);
            }
            Ok(len)
        })
    }

    /// Close the socket.
    pub fn shutdown(&self) -> NetResult<()> {
        SOCKET_SET.with_socket_mut::<udp::Socket, _, _>(self.handle, |socket| {
            debug!("UDP socket {}: shutting down", self.handle);
            socket.close();
        });
        SOCKET_SET.poll_interfaces();
        Ok(())
    }

    /// Whether the socket is readable or writable.
    pub fn poll(&self) -> NetResult<NetPollState> {
        if self.local_addr.read().is_none() {
            return Ok(NetPollState {
                readable: false,
                writable: false,
            });
        }
        SOCKET_SET.with_socket_mut::<udp::Socket, _, _>(self.handle, |socket| {
            Ok(NetPollState {
                readable: socket.can_recv(),
                writable: socket.can_send(),
            })
        })
    }
}

/// Private methods
impl UdpSocket {
    fn remote_endpoint(&self) -> NetResult<IpEndpoint> {
        match self.peer_addr.try_read() {
            Some(addr) => addr.ok_or(NetError::NotConnected),
            None => Err(NetError::NotConnected),
        }
    }

    fn send_impl(&self, buf: &[u8], remote_endpoint: IpEndpoint) -> NetResult<usize> {
        if self.local_addr.read().is_none() {
            warn!("UDP socket {}: send() failed: not bound", self.handle);
            // bound self to a random port
            self.bind(into_core_sockaddr(UNSPECIFIED_ENDPOINT))?;
            // return Err(NetError::NotConnected);
        }

        self.block_on(|| {
            SOCKET_SET.with_socket_mut::<udp::Socket, _, _>(self.handle, |socket| {
                if socket.can_send() {
                    socket
                        .send_slice(buf, remote_endpoint)
                        .map_err(|e| match e {
                            SendError::BufferFull => NetError::WouldBlock,
                            SendError::Unaddressable => {
                                warn!("UDP socket {}: send() failed: unaddressable", self.handle);
                                NetError::ConnectionRefused
                            }
                        })?;
                    Ok(buf.len())
                } else if !socket.is_open() {
                    error!("UDP socket {}: send() failed: not connected", self.handle);
                    Err(NetError::NotConnected)
                } else {
                    // tx buffer is full
                    Err(NetError::WouldBlock)
                }
            })
        })
    }

    fn recv_impl<F, T>(&self, mut op: F) -> NetResult<T>
    where
        F: FnMut(&mut udp::Socket) -> NetResult<T>,
    {
        if self.local_addr.read().is_none() {
            error!("UDP socket {}: recv() failed: not bound", self.handle);
            return Err(NetError::NotConnected);
        }

        self.block_on(|| {
            SOCKET_SET.with_socket_mut::<udp::Socket, _, _>(self.handle, |socket| {
                if socket.can_recv() {
                    // data available
                    op(socket)
                } else if !socket.is_open() {
                    error!("UDP socket {}: recv() failed: not connected", self.handle);
                    Err(NetError::NotConnected)
                } else {
                    error!("UDP socket {}: recv() failed: no data", socket.endpoint());
                    // no more data
                    Err(NetError::WouldBlock)
                }
            })
        })
    }

    fn block_on<F, T>(&self, mut f: F) -> NetResult<T>
    where
        F: FnMut() -> NetResult<T>,
    {
        if self.is_nonblocking() {
            f()
        } else {
            loop {
                SOCKET_SET.poll_interfaces();
                match f() {
                    Ok(t) => return Ok(t),
                    Err(NetError::WouldBlock) => {
                        let kernel_func = KERNEL_NET_FUNC.get().unwrap();
                        let has_signal = kernel_func.yield_now();
                        if !has_signal {
                            continue;
                        }
                        return Err(NetError::Interrupted);
                    }
                    Err(e) => return Err(e),
                }
            }
        }
    }
}

impl Drop for UdpSocket {
    fn drop(&mut self) {
        // delete reuse port
        self.shutdown().ok();
        SOCKET_SET.remove(self.handle);
    }
}

fn get_ephemeral_port() -> NetResult<u16> {
    const PORT_START: u16 = 0xc000;
    const PORT_END: u16 = 0xffff;
    static CURR: Mutex<u16> = Mutex::new(PORT_START);
    let mut curr = CURR.lock();

    let port = *curr;
    if *curr == PORT_END {
        *curr = PORT_START;
    } else {
        *curr += 1;
    }
    Ok(port)
}
