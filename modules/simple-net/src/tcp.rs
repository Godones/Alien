use core::cell::UnsafeCell;
use core::net::SocketAddr;
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use log::{debug, error, warn};
use smoltcp::iface::SocketHandle;
use smoltcp::socket::tcp::{self, ConnectError, State};
use smoltcp::wire::{IpEndpoint, IpListenEndpoint};

use kernel_sync::Mutex;

use crate::{KERNEL_NET_FUNC, LISTENING_TABLE, NET_INTERFACE};
use crate::common::{NetError, NetPollState, NetResult};

use super::{SOCKET_SET, SocketSetWrapper};
use super::addr::{from_core_sockaddr, into_core_sockaddr, is_unspecified, UNSPECIFIED_ENDPOINT};

// State transitions:
// CLOSED -(connect)-> BUSY -> CONNECTING -> CONNECTED -(shutdown)-> BUSY -> CLOSED
//       |
//       |-(listen)-> BUSY -> LISTENING -(shutdown)-> BUSY -> CLOSED
//       |
//        -(bind)-> BUSY -> CLOSED
const STATE_CLOSED: u8 = 0;
const STATE_BUSY: u8 = 1;
const STATE_CONNECTING: u8 = 2;
const STATE_CONNECTED: u8 = 3;
const STATE_LISTENING: u8 = 4;

/// A TCP socket that provides POSIX-like APIs.
///
/// - [`connect`] is for TCP clients.
/// - [`bind`], [`listen`], and [`accept`] are for TCP servers.
/// - Other methods are for both TCP clients and servers.
///
/// [`connect`]: TcpSocket::connect
/// [`bind`]: TcpSocket::bind
/// [`listen`]: TcpSocket::listen
/// [`accept`]: TcpSocket::accept
pub struct TcpSocket {
    state: AtomicU8,
    handle: UnsafeCell<Option<SocketHandle>>,
    local_addr: UnsafeCell<IpEndpoint>,
    peer_addr: UnsafeCell<IpEndpoint>,
    nonblock: AtomicBool,
}

unsafe impl Sync for TcpSocket {}

impl TcpSocket {
    /// Creates a new TCP socket.
    pub const fn new() -> Self {
        Self {
            state: AtomicU8::new(STATE_CLOSED),
            handle: UnsafeCell::new(None),
            local_addr: UnsafeCell::new(UNSPECIFIED_ENDPOINT),
            peer_addr: UnsafeCell::new(UNSPECIFIED_ENDPOINT),
            nonblock: AtomicBool::new(false),
        }
    }

    /// Creates a new TCP socket that is already connected.
    const fn new_connected(
        handle: SocketHandle,
        local_addr: IpEndpoint,
        peer_addr: IpEndpoint,
    ) -> Self {
        Self {
            state: AtomicU8::new(STATE_CONNECTED),
            handle: UnsafeCell::new(Some(handle)),
            local_addr: UnsafeCell::new(local_addr),
            peer_addr: UnsafeCell::new(peer_addr),
            nonblock: AtomicBool::new(false),
        }
    }

    /// Returns the local address and port, or
    /// [`Err(NotConnected)`](AxError::NotConnected) if not connected.
    #[inline]
    pub fn local_addr(&self) -> NetResult<SocketAddr> {
        match self.get_state() {
            STATE_CONNECTED | STATE_CLOSED | STATE_LISTENING => {
                Ok(into_core_sockaddr(unsafe { self.local_addr.get().read() }))
            }
            _ => Err(NetError::NotConnected),
        }
    }

    /// Returns the remote address and port, or
    /// [`Err(NotConnected)`](AxError::NotConnected) if not connected.
    #[inline]
    pub fn peer_addr(&self) -> NetResult<SocketAddr> {
        match self.get_state() {
            STATE_CONNECTED | STATE_LISTENING => {
                Ok(into_core_sockaddr(unsafe { self.peer_addr.get().read() }))
            }
            _ => Err(NetError::NotConnected),
        }
    }

    /// Returns whether this socket is in nonblocking mode.
    #[inline]
    pub fn is_nonblocking(&self) -> bool {
        self.nonblock.load(Ordering::Acquire)
    }

    /// Moves this TCP stream into or out of nonblocking mode.
    ///
    /// This will result in `read`, `write`, `recv` and `send` operations
    /// becoming nonblocking, i.e., immediately returning from their calls.
    /// If the IO operation is successful, `Ok` is returned and no further
    /// action is required. If the IO operation could not be completed and needs
    /// to be retried, an error with kind  [`Err(WouldBlock)`](AxError::WouldBlock) is
    /// returned.
    #[inline]
    pub fn set_nonblocking(&self, nonblocking: bool) {
        self.nonblock.store(nonblocking, Ordering::Release);
    }

    /// Connects to the given address and port.
    ///
    /// The local port is generated automatically.
    pub fn connect(&self, remote_addr: SocketAddr) -> NetResult<()> {
        self.update_state(STATE_CLOSED, STATE_CONNECTING, || {
            // SAFETY: no other threads can read or write these fields.

            // Create a new socket if not have one.
            let handle = unsafe { self.handle.get().read() }
                .unwrap_or_else(|| SOCKET_SET.add(SocketSetWrapper::new_tcp_socket()));

            // TODO: check remote addr unreachable
            let remote_endpoint = from_core_sockaddr(remote_addr);
            let bound_endpoint = self.bound_endpoint()?;
            let iface = NET_INTERFACE.get().unwrap().raw_interface();
            let (local_endpoint, remote_endpoint) = SOCKET_SET
                .with_socket_mut::<tcp::Socket, _, _>(handle, |socket| {
                    socket
                        .connect(iface.lock().context(), remote_endpoint, bound_endpoint)
                        .or_else(|e| match e {
                            ConnectError::InvalidState => {
                                warn!("socket connect() failed: invalid state");
                                Err(NetError::BadState)
                            }
                            ConnectError::Unaddressable => {
                                warn!("socket connect() failed: unaddressable");
                                Err(NetError::Unaddressable)
                            }
                        })?;
                    Ok((
                        socket.local_endpoint().unwrap(),
                        socket.remote_endpoint().unwrap(),
                    ))
                })?;
            unsafe {
                // SAFETY: no other threads can read or write these fields as we
                // have changed the state to `BUSY`.
                self.local_addr.get().write(local_endpoint);
                self.peer_addr.get().write(remote_endpoint);
                self.handle.get().write(Some(handle));
            }
            Ok(())
        })
            .unwrap_or_else(|e| {
                warn!("socket connect() failed: already connected,state :{}", e);
                Err(NetError::AlreadyExists)
            })?; // EISCONN

        // Here our state must be `CONNECTING`, and only one thread can run here.
        if self.is_nonblocking() {
            Err(NetError::WouldBlock)
        } else {
            self.block_on(|| {
                let NetPollState { writable, .. } = self.poll_connect()?;
                if !writable {
                    Err(NetError::WouldBlock)
                } else if self.get_state() == STATE_CONNECTED {
                    Ok(())
                } else {
                    warn!("socket connect() failed: connection refused");
                    Err(NetError::ConnectionRefused)
                }
            })
        }
    }

    /// Binds an unbound socket to the given address and port.
    ///
    /// If the given port is 0, it generates one automatically.
    ///
    /// It's must be called before [`listen`](Self::listen) and
    /// [`accept`](Self::accept).
    pub fn bind(&self, mut local_addr: SocketAddr) -> NetResult<()> {
        self.update_state(STATE_CLOSED, STATE_CLOSED, || {
            // TODO: check addr is available
            if local_addr.port() == 0 {
                local_addr.set_port(get_ephemeral_port()?);
            }
            // SAFETY: no other threads can read or write `self.local_addr` as we
            // have changed the state to `BUSY`.
            unsafe {
                let old = self.local_addr.get().read();
                if old != UNSPECIFIED_ENDPOINT {
                    warn!("socket bind() failed: already bound");
                    return Err(NetError::InvalidInput);
                }
                self.local_addr.get().write(from_core_sockaddr(local_addr));
            }
            Ok(())
        })
            .unwrap_or_else(|_| {
                warn!("socket bind() failed: already bound");
                Err(NetError::InvalidInput)
            })
    }

    /// Starts listening on the bound address and port.
    ///
    /// It's must be called after [`bind`](Self::bind) and before
    /// [`accept`](Self::accept).
    pub fn listen(&self) -> NetResult<()> {
        self.update_state(STATE_CLOSED, STATE_LISTENING, || {
            let bound_endpoint = self.bound_endpoint()?;
            unsafe {
                (*self.local_addr.get()).port = bound_endpoint.port;
            }
            LISTENING_TABLE.listen(bound_endpoint)?;
            debug!("TCP socket listening on {}", bound_endpoint);
            Ok(())
        })
            .unwrap_or(Ok(())) // ignore simultaneous `listen`s.
    }

    /// Accepts a new connection.
    ///
    /// This function will block the calling thread until a new TCP connection
    /// is established. When established, a new [`TcpSocket`] is returned.
    ///
    /// It's must be called after [`bind`](Self::bind) and [`listen`](Self::listen).
    pub fn accept(&self) -> NetResult<TcpSocket> {
        if !self.is_listening() {
            warn!("socket accept() failed: not listen");
            return Err(NetError::InvalidInput);
        }

        // SAFETY: `self.local_addr` should be initialized after `bind()`.
        let local_port = unsafe { self.local_addr.get().read().port };
        self.block_on(|| {
            let (handle, (local_addr, peer_addr)) = LISTENING_TABLE.accept(local_port)?;
            error!("TCP socket accepted a new connection {}", peer_addr);
            Ok(TcpSocket::new_connected(handle, local_addr, peer_addr))
        })
    }

    /// Close the connection.
    pub fn shutdown(&self) -> NetResult<()> {
        // stream
        self.update_state(STATE_CONNECTED, STATE_CLOSED, || {
            // SAFETY: `self.handle` should be initialized in a connected socket, and
            // no other threads can read or write it.
            let handle = unsafe { self.handle.get().read().unwrap() };
            SOCKET_SET.with_socket_mut::<tcp::Socket, _, _>(handle, |socket| {
                debug!("TCP socket {}: shutting down", handle);
                socket.close();
            });
            unsafe { self.local_addr.get().write(UNSPECIFIED_ENDPOINT) }; // clear bound address
            SOCKET_SET.poll_interfaces();
            Ok(())
        })
            .unwrap_or(Ok(()))?;

        // listener
        self.update_state(STATE_LISTENING, STATE_CLOSED, || {
            // SAFETY: `self.local_addr` should be initialized in a listening socket,
            // and no other threads can read or write it.
            let local_port = unsafe { self.local_addr.get().read().port };
            unsafe { self.local_addr.get().write(UNSPECIFIED_ENDPOINT) }; // clear bound address
            LISTENING_TABLE.unlisten(local_port);
            SOCKET_SET.poll_interfaces();
            Ok(())
        })
            .unwrap_or(Ok(()))?;

        // ignore for other states
        Ok(())
    }

    /// Receives data from the socket, stores it in the given buffer.
    pub fn recv(&self, buf: &mut [u8]) -> NetResult<usize> {
        if self.is_connecting() {
            return Err(NetError::WouldBlock);
        } else if !self.is_connected() {
            warn!("socket recv() failed: not connected");
            return Err(NetError::NotConnected);
        }

        // SAFETY: `self.handle` should be initialized in a connected socket.
        let handle = unsafe { self.handle.get().read().unwrap() };
        self.block_on(|| {
            SOCKET_SET.with_socket_mut::<tcp::Socket, _, _>(handle, |socket| {
                if !socket.is_active() {
                    // not open
                    warn!("socket recv() failed: not open");
                    Err(NetError::NotConnected)
                } else if !socket.may_recv() {
                    // connection closed
                    Ok(0)
                } else if socket.recv_queue() > 0 {
                    // data available
                    // TODO: use socket.recv(|buf| {...})
                    let len = socket.recv_slice(buf).map_err(|_| {
                        warn!("socket recv() failed: bad state");
                        NetError::BadState
                    })?;
                    Ok(len)
                } else {
                    // no more data
                    Err(NetError::WouldBlock)
                }
            })
        })
    }

    /// Transmits data in the given buffer.
    pub fn send(&self, buf: &[u8]) -> NetResult<usize> {
        if self.is_connecting() {
            return Err(NetError::WouldBlock);
        } else if !self.is_connected() {
            warn!("socket send() failed: not connected");
            return Err(NetError::NotConnected);
        }

        // SAFETY: `self.handle` should be initialized in a connected socket.
        let handle = unsafe { self.handle.get().read().unwrap() };
        self.block_on(|| {
            SOCKET_SET.with_socket_mut::<tcp::Socket, _, _>(handle, |socket| {
                if !socket.is_active() || !socket.may_send() {
                    // closed by remote
                    warn!("socket send() failed: connection reset");
                    Err(NetError::ConnectionReset)
                } else if socket.can_send() {
                    // connected, and the tx buffer is not full
                    // TODO: use socket.send(|buf| {...})
                    let len = socket.send_slice(buf).map_err(|_| {
                        warn!("socket recv() failed: bad state");
                        NetError::BadState
                    })?;
                    Ok(len)
                } else {
                    // tx buffer is full
                    Err(NetError::WouldBlock)
                }
            })
        })
    }

    /// Whether the socket is readable or writable.
    pub fn poll(&self) -> NetResult<NetPollState> {
        warn!("socket state: {:?}", self.get_state());
        match self.get_state() {
            STATE_CONNECTING => self.poll_connect(),
            STATE_CONNECTED | STATE_CLOSED => self.poll_stream(),
            STATE_LISTENING => self.poll_listener(),
            _ => Ok(NetPollState {
                readable: false,
                writable: false,
            }),
        }
    }
}

/// Private methods
impl TcpSocket {
    #[inline]
    fn get_state(&self) -> u8 {
        self.state.load(Ordering::Acquire)
    }

    #[inline]
    fn set_state(&self, state: u8) {
        self.state.store(state, Ordering::Release);
    }

    /// Update the state of the socket atomically.
    ///
    /// If the current state is `expect`, it first changes the state to `STATE_BUSY`,
    /// then calls the given function. If the function returns `Ok`, it changes the
    /// state to `new`, otherwise it changes the state back to `expect`.
    ///
    /// It returns `Ok` if the current state is `expect`, otherwise it returns
    /// the current state in `Err`.
    fn update_state<F, T>(&self, expect: u8, new: u8, f: F) -> Result<NetResult<T>, u8>
        where
            F: FnOnce() -> NetResult<T>,
    {
        match self
            .state
            .compare_exchange(expect, STATE_BUSY, Ordering::Acquire, Ordering::Acquire)
        {
            Ok(_) => {
                let res = f();
                if res.is_ok() {
                    self.set_state(new);
                } else {
                    self.set_state(expect);
                }
                Ok(res)
            }
            Err(old) => Err(old),
        }
    }

    #[inline]
    fn is_connecting(&self) -> bool {
        self.get_state() == STATE_CONNECTING
    }

    #[inline]
    fn is_connected(&self) -> bool {
        self.get_state() == STATE_CONNECTED
    }

    #[inline]
    fn is_listening(&self) -> bool {
        self.get_state() == STATE_LISTENING
    }

    fn bound_endpoint(&self) -> NetResult<IpListenEndpoint> {
        // SAFETY: no other threads can read or write `self.local_addr`.
        let local_addr = unsafe { self.local_addr.get().read() };
        let port = if local_addr.port != 0 {
            local_addr.port
        } else {
            get_ephemeral_port()?
        };
        assert_ne!(port, 0);
        let addr = if !is_unspecified(local_addr.addr) {
            Some(local_addr.addr)
        } else {
            None
        };
        Ok(IpListenEndpoint { addr, port })
    }

    fn poll_connect(&self) -> NetResult<NetPollState> {
        // SAFETY: `self.handle` should be initialized above.
        let handle = unsafe { self.handle.get().read().unwrap() };
        let writable =
            SOCKET_SET.with_socket::<tcp::Socket, _, _>(handle, |socket| match socket.state() {
                State::SynSent => false, // wait for connection
                State::Established => {
                    self.set_state(STATE_CONNECTED);
                    // connected
                    warn!(
                        "TCP socket {}: connected to {}",
                        handle,
                        socket.remote_endpoint().unwrap(),
                    );
                    true
                }
                _ => {
                    unsafe {
                        self.local_addr.get().write(UNSPECIFIED_ENDPOINT);
                        self.peer_addr.get().write(UNSPECIFIED_ENDPOINT);
                    }
                    self.set_state(STATE_CLOSED); // connection failed
                    true
                }
            });
        warn!("poll_connect: writable = {}", writable);
        Ok(NetPollState {
            readable: false,
            writable,
        })
    }

    fn poll_stream(&self) -> NetResult<NetPollState> {
        // SAFETY: `self.handle` should be initialized in a connected socket.
        let handle = unsafe { self.handle.get().read().unwrap() };
        SOCKET_SET.with_socket::<tcp::Socket, _, _>(handle, |socket| {
            Ok(NetPollState {
                readable: !socket.may_recv() || socket.can_recv(),
                writable: !socket.may_send() || socket.can_send(),
            })
        })
    }

    fn poll_listener(&self) -> NetResult<NetPollState> {
        // SAFETY: `self.local_addr` should be initialized in a listening socket.
        let local_addr = unsafe { self.local_addr.get().read() };
        Ok(NetPollState {
            readable: LISTENING_TABLE.can_accept(local_addr.port)?,
            writable: false,
        })
    }

    /// Block the current thread until the given function completes or fails.
    ///
    /// If the socket is non-blocking, it calls the function once and returns
    /// immediately. Otherwise, it may call the function multiple times if it
    /// returns [`Err(WouldBlock)`](AxError::WouldBlock).
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

impl Drop for TcpSocket {
    fn drop(&mut self) {
        self.shutdown().ok();
        // Safe because we have mut reference to `self`.
        if let Some(handle) = unsafe { self.handle.get().read() } {
            SOCKET_SET.remove(handle);
        }
    }
}

fn get_ephemeral_port() -> NetResult<u16> {
    const PORT_START: u16 = 0xc000;
    const PORT_END: u16 = 0xffff;
    static CURR: Mutex<u16> = Mutex::new(PORT_START);

    let mut curr = CURR.lock();
    let mut tries = 0;
    // TODO: more robust
    while tries <= PORT_END - PORT_START {
        let port = *curr;
        if *curr == PORT_END {
            *curr = PORT_START;
        } else {
            *curr += 1;
        }
        if LISTENING_TABLE.can_listen(port) {
            return Ok(port);
        }
        tries += 1;
    }
    warn!("no avaliable ports!");
    Err(NetError::AddrInUse)
}
