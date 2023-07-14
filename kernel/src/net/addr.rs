#[derive(Debug)]
pub enum Addr {
    /// ip 地址+端口
    Ipv4(u32, u16),
    /// 初始化
    Empty,
    /// 未知
    Unknown,
}

#[repr(C)]
pub struct IpAddr {
    pub family: u16,
    pub port: u16,
    pub addr: u32,
}

pub const FAMILY_UNIX: u16 = 1;
pub const FAMILY_INTERNET: u16 = 2;

pub const LOCAL_LOOPBACK_ADDR: u32 = 0x7f000001;

// 地址解析
pub fn addr_resolution(family_user_addr: *const u16) -> Addr {
    let family = unsafe { *family_user_addr };
    match family {
        FAMILY_INTERNET => {
            let ip_addr = unsafe { &*(family_user_addr as *const IpAddr) };
            Addr::Ipv4(u32::from_be(ip_addr.addr), u16::from_be(ip_addr.port))
        }
        _ => Addr::Unknown,
    }
}
