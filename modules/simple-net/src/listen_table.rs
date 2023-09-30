use alloc::{boxed::Box, collections::VecDeque};
use core::ops::{Deref, DerefMut};

use log::{debug, error, warn};
use smoltcp::iface::{SocketHandle, SocketSet};
use smoltcp::socket::tcp::{self, State};
use smoltcp::wire::{IpAddress, IpEndpoint, IpListenEndpoint};

use crate::common::{NetError, NetResult, LISTEN_QUEUE_SIZE};
use kernel_sync::TicketMutex as Mutex;

use super::{SocketSetWrapper, SOCKET_SET};

const PORT_NUM: usize = 65536;

struct ListenTableEntry {
    listen_endpoint: IpListenEndpoint,
    syn_queue: VecDeque<SocketHandle>,
}

impl ListenTableEntry {
    pub fn new(listen_endpoint: IpListenEndpoint) -> Self {
        Self {
            listen_endpoint,
            syn_queue: VecDeque::with_capacity(LISTEN_QUEUE_SIZE),
        }
    }

    #[inline]
    fn can_accept(&self, dst: IpAddress) -> bool {
        match self.listen_endpoint.addr {
            Some(addr) => addr == dst,
            None => true,
        }
    }
}

impl Drop for ListenTableEntry {
    fn drop(&mut self) {
        for &handle in &self.syn_queue {
            SOCKET_SET.remove(handle);
        }
    }
}

pub struct ListenTable {
    tcp: Box<[Mutex<Option<Box<ListenTableEntry>>>]>,
}

impl ListenTable {
    pub fn new() -> Self {
        let tcp = unsafe {
            let mut buf = Box::new_uninit_slice(PORT_NUM);
            for i in 0..PORT_NUM {
                buf[i].write(Mutex::new(None));
            }
            buf.assume_init()
        };
        Self { tcp }
    }

    /// Check if the port is available for listening.
    pub fn can_listen(&self, port: u16) -> bool {
        self.tcp[port as usize].lock().is_none()
    }

    /// Listen on a port.
    ///
    /// Create a new `ListenTableEntry` and store it in the table.
    pub fn listen(&self, listen_endpoint: IpListenEndpoint) -> NetResult<()> {
        let port = listen_endpoint.port;
        assert_ne!(port, 0);
        let mut entry = self.tcp[port as usize].lock();
        if entry.is_none() {
            *entry = Some(Box::new(ListenTableEntry::new(listen_endpoint)));
            Ok(())
        } else {
            warn!("socket listen() failed: port {} is in use", port);
            Err(NetError::AddrInUse)
        }
    }

    /// Unlisten on a port.
    pub fn unlisten(&self, port: u16) {
        debug!("TCP socket unlisten on {}", port);
        *self.tcp[port as usize].lock() = None;
    }

    /// Check whether the port can accept a connection.
    ///
    /// Return `true` if the port is listening and there is at least one connection in the SYN queue.
    pub fn can_accept(&self, port: u16) -> NetResult<bool> {
        if let Some(entry) = self.tcp[port as usize].lock().deref() {
            Ok(entry.syn_queue.iter().any(|&handle| is_connected(handle)))
        } else {
            // ax_err!(InvalidInput, "socket accept() failed: not listen")
            warn!("socket accept() failed: not listen");
            Err(NetError::InvalidInput)
        }
    }

    // The accept() system call is used with connection-based socket
    // types (SOCK_STREAM, SOCK_SEQPACKET).  It extracts the first
    // connection request on the queue of pending connections for the
    // listening socket, sockfd, creates a new connected socket, and
    // returns a new file descriptor referring to that socket.  The
    // newly created socket is not in the listening state.  The original
    // socket sockfd is unaffected by this call.

    /// Accept a connection.
    pub fn accept(&self, port: u16) -> NetResult<(SocketHandle, (IpEndpoint, IpEndpoint))> {
        if let Some(entry) = self.tcp[port as usize].lock().deref_mut() {
            let syn_queue = &mut entry.syn_queue;
            let (idx, addr_tuple) = syn_queue
                .iter()
                .enumerate()
                .find_map(|(idx, &handle)| {
                    is_connected(handle).then(|| (idx, get_addr_tuple(handle)))
                })
                .ok_or(NetError::WouldBlock)?; // wait for connection
            if idx > 0 {
                warn!(
                    "slow SYN queue enumeration: index = {}, len = {}!",
                    idx,
                    syn_queue.len()
                );
            }
            let handle = syn_queue.swap_remove_front(idx).unwrap();
            Ok((handle, addr_tuple))
        } else {
            warn!("socket accept() failed: not listen");
            Err(NetError::InvalidInput)
        }
    }

    pub fn incoming_tcp_packet(
        &self,
        src: IpEndpoint,
        dst: IpEndpoint,
        sockets: &mut SocketSet<'_>,
    ) {
        if let Some(entry) = self.tcp[dst.port as usize].lock().deref_mut() {
            if !entry.can_accept(dst.addr) {
                // not listening on this address
                return;
            }
            if entry.syn_queue.len() >= LISTEN_QUEUE_SIZE {
                // SYN queue is full, drop the packet
                warn!("SYN queue overflow!");
                return;
            }
            let mut socket = SocketSetWrapper::new_tcp_socket();
            if socket.listen(entry.listen_endpoint).is_ok() {
                let handle = sockets.add(socket);
                debug!(
                    "TCP socket {}: prepare for connection {} -> {}",
                    handle, src, entry.listen_endpoint
                );
                entry.syn_queue.push_back(handle);
            }
        }
    }
}

fn is_connected(handle: SocketHandle) -> bool {
    SOCKET_SET.with_socket::<tcp::Socket, _, _>(handle, |socket| {
        error!("[is_connected] socket state: {:?}", socket.state());
        !(socket.state() == State::Listen || socket.state() == State::SynReceived)
    })
}

fn get_addr_tuple(handle: SocketHandle) -> (IpEndpoint, IpEndpoint) {
    SOCKET_SET.with_socket::<tcp::Socket, _, _>(handle, |socket| {
        (
            socket.local_endpoint().unwrap(),
            socket.remote_endpoint().unwrap(),
        )
    })
}
