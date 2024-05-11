use alloc::{sync::Arc, vec, vec::Vec};
use core::{cmp, net::SocketAddrV4};

use constants::{
    io::PollEvents,
    net::{Domain, SocketType},
    AlienError, AlienResult,
};
use ksync::Mutex;
use lose_net_stack::{connection::NetServer, net_trait::SocketInterface};

use crate::{nic::NetMod, SocketFile};

#[derive(Clone)]
pub struct SocketOptions {
    pub wsize: usize,
    pub rsize: usize,
}

#[allow(dead_code)]
pub struct Socket {
    pub domain: Domain,
    pub net_type: SocketType,
    pub inner: Arc<dyn SocketInterface>,
    pub options: Mutex<SocketOptions>,
    pub buf: Mutex<Vec<u8>>,
}

impl Drop for Socket {
    fn drop(&mut self) {
        log::debug!("strong count: {}", Arc::strong_count(&self.inner));
        // TIPS: the socke table map will consume a strong reference.
        if !self.inner.is_closed().unwrap()
            && (Arc::strong_count(&self.inner) == 2 || Arc::strong_count(&self.inner) == 1)
        {
            log::info!("drop socket");
            // self.inner.close().expect("cant close socket when droping socket in os.");
            let _ = self.inner.close();
        }
        // self.inner.close();
    }
}

impl Socket {
    pub fn new(
        net_server: &Arc<NetServer<NetMod>>,
        domain: Domain,
        net_type: SocketType,
    ) -> Arc<Self> {
        let inner: Arc<dyn SocketInterface> = match net_type {
            SocketType::SOCK_STREAM => net_server.blank_tcp(),
            SocketType::SOCK_DGRAM => net_server.blank_udp(),
            _ => {
                panic!("can't create raw socket")
            }
        };
        Arc::new(Self {
            domain,
            net_type,
            inner,
            options: Mutex::new(SocketOptions { wsize: 0, rsize: 0 }),
            buf: Mutex::new(vec![]),
        })
    }

    pub fn recv_from(&self) -> AlienResult<(Vec<u8>, SocketAddrV4)> {
        log::trace!("{} try to recv data", self.inner.get_local().unwrap());
        match self.inner.recv_from() {
            Ok((data, remote)) => Ok((data, remote)),
            Err(_err) => Err(AlienError::EBLOCKING),
        }
    }

    pub fn new_with_inner(
        domain: Domain,
        net_type: SocketType,
        inner: Arc<dyn SocketInterface>,
    ) -> Arc<Self> {
        Arc::new(Self {
            domain,
            net_type,
            inner,
            options: Mutex::new(SocketOptions { wsize: 0, rsize: 0 }),
            buf: Mutex::new(vec![]),
        })
    }

    pub fn reuse(&self, net_server: &Arc<NetServer<NetMod>>, port: u16) -> Self {
        // NET_SERVER.get_tcp(port)
        match self.inner.get_protocol().unwrap() {
            lose_net_stack::connection::SocketType::TCP => {
                if let Some(socket_inner) = net_server.get_tcp(&port) {
                    Self {
                        domain: self.domain,
                        net_type: self.net_type,
                        inner: socket_inner,
                        options: Mutex::new(self.options.lock().clone()),
                        buf: Mutex::new(vec![]),
                    }
                } else {
                    unreachable!("can't reusetcp in blank tcp")
                }
            }
            lose_net_stack::connection::SocketType::UDP => {
                if let Some(socket_inner) = net_server.get_udp(&port) {
                    Self {
                        domain: self.domain,
                        net_type: self.net_type,
                        inner: socket_inner,
                        options: Mutex::new(self.options.lock().clone()),
                        buf: Mutex::new(vec![]),
                    }
                } else {
                    unreachable!("can't reusetcp in blank udp")
                }
            }
            lose_net_stack::connection::SocketType::RAW => todo!(),
        }
    }
}

impl SocketFile for Socket {
    fn write_at(&self, _offset: usize, buffer: &[u8]) -> AlienResult<usize> {
        match self.inner.sendto(&buffer, None) {
            Ok(len) => {
                self.options.lock().wsize += len;
                Ok(len)
            }
            Err(_err) => Err(AlienError::EACCES),
        }
    }

    fn read_at(&self, _offset: usize, buffer: &mut [u8]) -> AlienResult<usize> {
        let mut data = self.buf.lock().clone();
        if data.len() == 0 {
            match self.inner.recv_from() {
                Ok((recv_data, _)) => {
                    data = recv_data;
                }
                Err(_err) => return Err(AlienError::EBLOCKING),
            }
        }
        let rlen = cmp::min(data.len(), buffer.len());
        buffer[..rlen].copy_from_slice(&data[..rlen]);
        self.options.lock().rsize += rlen;
        if buffer.len() == 1 {
            // DebugConsole::putchar(buffer[0]);
        }
        if rlen < data.len() {
            *self.buf.lock() = data[rlen..].to_vec();
        } else {
            self.buf.lock().clear();
        }
        Ok(rlen)
    }

    fn poll(&self, events: PollEvents) -> AlienResult<PollEvents> {
        let mut res = PollEvents::empty();
        if events.contains(PollEvents::OUT)
            && !self.inner.is_closed().unwrap()
            && self.inner.get_remote().is_ok()
        {
            res |= PollEvents::OUT;
        }
        if self.inner.readable().unwrap() && events.contains(PollEvents::IN) {
            res |= PollEvents::IN;
        }
        Ok(res)
    }
}
