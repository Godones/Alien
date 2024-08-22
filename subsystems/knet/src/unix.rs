//! 有关 Unix 协议族下的套接字结构。(目前有关的功能有待支持)
use alloc::{
    string::String,
    sync::{Arc, Weak},
    vec::Vec,
};

use constants::{AlienResult, LinuxErrno};
use ksync::Mutex;

use crate::socket::{Socket, SocketFile, SocketFileExt};

/// Unix 协议族下的套接字结构
#[allow(unused)]
pub struct UnixSocket {
    inner: Mutex<UnixSocketInner>,
}

struct UnixSocketInner {
    remote: Weak<SocketFile>,
    buf: Vec<u8>,
}

impl UnixSocket {
    /// 创建一个新的 Unix 协议族下的套接字结构
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(UnixSocketInner {
                remote: Weak::<SocketFile>::new(),
                buf: Vec::new(),
            }),
        }
    }

    /// UnixSocket 的 connect 操作
    pub fn connect(&self, _file_path: String) -> AlienResult<()> {
        Err(LinuxErrno::ENOENT)
    }

    pub fn set_remote(&self, remote: &Arc<SocketFile>) {
        self.inner.lock().remote = Arc::downgrade(remote);
    }

    pub fn send_to(&self, buf: &[u8]) -> AlienResult<usize> {
        if let Some(remote) = self.inner.lock().remote.upgrade() {
            let socket_guard = remote.get_socketdata()?;
            match socket_guard.socket {
                Socket::Unix(ref unix_socket) => {
                    unix_socket.inner.lock().buf.extend_from_slice(buf);
                    Ok(buf.len())
                }
                _ => Err(LinuxErrno::EINVAL),
            }
        } else {
            Err(LinuxErrno::ENOTCONN)
        }
    }

    pub fn ready_read(&self) -> bool {
        self.inner.lock().buf.len() > 0
    }

    pub fn recvfrom(&self, buf: &mut [u8]) -> AlienResult<usize> {
        let inner_buf = &mut self.inner.lock().buf;
        if inner_buf.len() > 0 {
            let len = inner_buf.len().min(buf.len());
            buf[..len].copy_from_slice(&inner_buf[..len]);
            inner_buf.drain(..len);
            Ok(len)
        } else {
            Err(LinuxErrno::EAGAIN)
        }
    }
}
