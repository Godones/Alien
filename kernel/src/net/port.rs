use alloc::string::String;
use alloc::sync::Arc;
use alloc::{collections::BTreeMap, vec::Vec};

use kernel_sync::Mutex;

use crate::fs::file::KFile;
use crate::net::addr::IpAddr;

/// 端口映射
static PORT_MAP: Mutex<BTreeMap<u16, PortData>> = Mutex::new(BTreeMap::new());
static SOCKET_SHARED_FILE: Mutex<BTreeMap<String, Arc<KFile>>> = Mutex::new(BTreeMap::new());
static PORT2IP: Mutex<BTreeMap<u16, IpAddr>> = Mutex::new(BTreeMap::new());

/// 端口上的被发送或等待接收的数据
pub struct PortData {
    src_port: u16,
    data: Mutex<Vec<u8>>,
}

impl PortData {
    /// 建立新的端口映射
    pub fn new(src_port: u16) -> Self {
        Self {
            src_port,
            data: Mutex::new(Vec::new()),
        }
    }
    /// 读数据到 buf 中
    pub fn read(&self, buf: &mut [u8]) -> Option<usize> {
        let mut data = self.data.lock();
        let read_len = if data.len() < buf.len() {
            data.len()
        } else {
            buf.len()
        };
        buf[..read_len].copy_from_slice(&data[..read_len]);
        *data = data.split_off(read_len);
        Some(read_len)
    }
    /// 从 buf 写入数据
    pub fn write(&self, buf: &[u8]) -> Option<usize> {
        let mut data = self.data.lock();
        data.extend_from_slice(buf);
        Some(buf.len())
    }

    pub fn src_port(&self) -> u16 {
        self.src_port
    }
}

pub fn read_from_port(port: u16, buf: &mut [u8]) -> Option<(usize, u16)> {
    let map = PORT_MAP.lock();
    match map.get(&port) {
        Some(data) => data.read(buf).map(|len| (len, data.src_port())),
        None => {
            // 端口还没有数据
            None
        }
    }
}

pub fn write_to_port(src_port: u16, port: u16, buf: &[u8]) -> Option<usize> {
    let mut map = PORT_MAP.lock();
    match map.get(&port) {
        Some(data) => data.write(buf),
        None => {
            // 新建端口数据
            let port_data = PortData::new(src_port);
            let write_len = port_data.write(buf);
            map.insert(port, port_data);
            write_len
        }
    }
}

pub fn delete_port(port: u16) {
    let mut map = PORT_MAP.lock();
    map.remove(&port);
}

pub fn alloc_ephemeral_port() -> Option<u16> {
    let map = PORT2IP.lock();
    (5555..65535)
        .find(|port| !map.contains_key(port))
        .map(|x| Some(x))
        .unwrap_or(None)
}

pub fn check_socket_file_exist(name: &str) -> bool {
    let map = SOCKET_SHARED_FILE.lock();
    map.contains_key(name)
}

pub fn insert_port2ip(port: u16, ip: IpAddr) {
    let mut map = PORT2IP.lock();
    map.insert(port, ip);
}

pub fn delete_port2ip(port: u16) {
    let mut map = PORT2IP.lock();
    map.remove(&port);
}

pub fn find_ip_by_port(port: u16) -> Option<IpAddr> {
    let map = PORT2IP.lock();
    map.get(&port).map(|x| x.clone())
}
