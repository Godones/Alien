use alloc::string::String;
use alloc::sync::Arc;
use alloc::{collections::BTreeMap, vec::Vec};

use kernel_sync::Mutex;

use crate::config::MAX_SOCKET_DATA_LEN;
use crate::fs::file::KFile;
use crate::net::addr::IpAddr;

/// 端口映射
static PORT_MAP: Mutex<BTreeMap<u16, PortData>> = Mutex::new(BTreeMap::new());
static SOCKET_SHARED_FILE: Mutex<BTreeMap<String, Arc<KFile>>> = Mutex::new(BTreeMap::new());
static PORT2IP: Mutex<BTreeMap<u16, IpAddr>> = Mutex::new(BTreeMap::new());

/// (src_ip, dst_ip) -> port
static SERVER_CLIENT_MAP: Mutex<BTreeMap<(IpAddr, IpAddr), PortData>> = Mutex::new(BTreeMap::new());
/// When client connect to server, client will mark the connection as false and wait for server to mark it as true.
static CONNECT_MAP: Mutex<BTreeMap<(IpAddr, IpAddr), bool>> = Mutex::new(BTreeMap::new());

/// 端口上的被发送或等待接收的数据
pub struct PortData {
    src_port: u16,
    data: Vec<Vec<u8>>,
}

impl PortData {
    /// 建立新的端口映射
    pub fn new(src_port: u16) -> Self {
        Self {
            src_port,
            data: Vec::new(),
        }
    }
    /// 读数据到 buf 中
    pub fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        let data = &mut self.data;
        let mut read_len = 0;
        let mut need_delete = Vec::new();
        for i in 0..data.len() {
            let len = data[i].len();
            let min_len = core::cmp::min(len, buf.len() - read_len);
            buf[read_len..read_len + min_len].copy_from_slice(&data[i][0..min_len]);
            read_len += min_len;
            if min_len == len {
                need_delete.push(i);
            } else {
                data[i] = data[i][min_len..].to_vec();
            }
            if read_len == buf.len() {
                break;
            }
        }
        for i in need_delete.iter().rev() {
            data.remove(*i);
        }
        Some(read_len)
    }
    /// 从 buf 写入数据
    pub fn write(&mut self, buf: &[u8]) -> Option<usize> {
        self.data.push(buf.to_vec());
        Some(buf.len())
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn indeed_len(&self) -> usize {
        self.data.iter().map(|x| x.len()).sum()
    }

    pub fn src_port(&self) -> u16 {
        self.src_port
    }
}

pub fn read_from_port_with_port_map(port: u16, buf: &mut [u8]) -> Option<(usize, u16)> {
    let mut map = PORT_MAP.lock();
    match map.get_mut(&port) {
        Some(data) => {
            if data.len() > 0 {
                data.read(buf).map(|len| (len, data.src_port()))
            } else {
                None
            }
        }
        None => None,
    }
}

pub fn write_to_port_with_port_map(src_port: u16, port: u16, buf: &[u8]) -> Option<usize> {
    let mut map = PORT_MAP.lock();
    match map.get_mut(&port) {
        Some(data) => data.write(buf),
        None => {
            // 新建端口数据
            let mut port_data = PortData::new(src_port);
            let write_len = port_data.write(buf);
            map.insert(port, port_data);
            write_len
        }
    }
}

pub fn delete_port_with_port_map(port: u16) {
    let mut map = PORT_MAP.lock();
    map.remove(&port);
}

pub fn check_port_have_data_with_port_map(port: u16) -> bool {
    let map = PORT_MAP.lock();
    let data = map.get(&port);
    data.is_some() && data.unwrap().len() > 0
}

pub fn check_port_can_write_with_port_map(port: u16) -> bool {
    let map = PORT_MAP.lock();
    let data = map.get(&port);
    data.is_some() && data.unwrap().indeed_len() < MAX_SOCKET_DATA_LEN
}

/// called by server, the port is server port
pub fn read_from_port_with_s_c_map(port: u16, buf: &mut [u8]) -> Option<(usize, u16)> {
    let server_ip = PORT2IP.lock().get(&port).unwrap().clone();
    let mut map = SERVER_CLIENT_MAP.lock();
    let res = map
        .iter_mut()
        .find(|((s_ip, _c_ip), data)| {
            if s_ip.clone() == server_ip && data.len() > 0 {
                return true;
            }
            false
        })
        .map(|((s_ip, c_ip), data)| {
            warn!(
                "read_from_port_with_s_c_map: server_ip: {:?}, c_ip: {:?}",
                s_ip, c_ip
            );
            data.read(buf).map(|len| (len, data.src_port()))
        });
    if res.is_some() {
        res.unwrap()
    } else {
        None
    }
}

/// Only client send to data use SERVER_CLIENT_MAP
pub fn write_to_port_with_s_c_map(src_port: u16, port: u16, buf: &[u8]) -> Option<usize> {
    let server_ip = PORT2IP.lock().get(&port).unwrap().clone();
    let src_ip = PORT2IP.lock().get(&src_port).unwrap().clone();
    let mut map = SERVER_CLIENT_MAP.lock();
    warn!(
        "write_to_port_with_s_c_map: server_ip: {:?}, src_ip: {:?}, map_len:{}",
        server_ip,
        src_ip,
        map.len()
    );
    match map.get_mut(&(server_ip.clone(), src_ip.clone())) {
        Some(data) => data.write(buf),
        None => {
            // 新建端口数据
            let mut port_data = PortData::new(src_port);
            let write_len = port_data.write(buf);
            map.insert((server_ip, src_ip), port_data);
            write_len
        }
    }
}

/// Only client close socket use this function
pub fn delete_port_with_s_c_map(port: u16) {
    warn!("delete_port_with_s_c_map: port: {:?}", port);
    let client_ip = PORT2IP.lock().get(&port).unwrap().clone();
    let mut server_client_map = SERVER_CLIENT_MAP.lock();
    let key = server_client_map.iter().find(|(k, _)| k.1 == client_ip);
    if let Some((k, _)) = key {
        let key = k.clone();
        server_client_map.remove(&key);
    }
}

/// called by server, the port is server port
pub fn check_port_have_data_with_s_c_map(port: u16) -> bool {
    let ip = PORT2IP.lock().get(&port).unwrap().clone();
    SERVER_CLIENT_MAP
        .lock()
        .iter()
        .find(|((s_ip, _c_ip), data)| {
            if s_ip.clone() == ip && data.len() > 0 {
                return true;
            }
            false
        })
        .is_some()
}

/// called by client
pub fn check_port_can_write_with_s_c_map(port: u16) -> bool {
    let ip = PORT2IP.lock().get(&port).unwrap().clone();
    SERVER_CLIENT_MAP
        .lock()
        .iter()
        .find(|((_s_ip, c_ip), data)| {
            if c_ip.clone() == ip && data.indeed_len() < MAX_SOCKET_DATA_LEN {
                return true;
            }
            false
        })
        .is_some()
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

pub fn create_connect_map(src_ip: IpAddr, dst_ip: IpAddr) {
    let mut map = CONNECT_MAP.lock();
    map.insert((src_ip, dst_ip), false);
}

pub fn mark_connect_map(src_ip: IpAddr, dst_ip: IpAddr) {
    let mut map = CONNECT_MAP.lock();
    let value = map.get_mut(&(src_ip, dst_ip)).unwrap();
    *value = true;
}

pub fn delete_connect_map(src_ip: IpAddr, dst_ip: IpAddr) {
    let mut map = CONNECT_MAP.lock();
    map.remove(&(src_ip, dst_ip));
}

pub fn check_connect_map(src_ip: IpAddr, dst_ip: IpAddr) -> bool {
    let map = CONNECT_MAP.lock();
    let value = map.get(&(src_ip, dst_ip)).unwrap();
    *value
}
