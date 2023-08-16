use syscall_table::syscall_func;

use crate::task::current_task;

/// 记录系统信息的结构，包括操作系统名、在网络中的用户名、操作系统release和version版本、硬件类型、域名等信息。
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Utsname {
    /// 操作系统名
    sysname: [u8; 65],
    /// Name within communications network to which the node is attached
    nodename: [u8; 65],
    /// 系统发行版
    release: [u8; 65],
    /// 系统版本
    version: [u8; 65],
    /// 硬件类型
    machine: [u8; 65],
    /// 域名
    domainname: [u8; 65],
}

/// 返回系统信息，信息保存在[`Utsname`]结构中。
fn system_info() -> Utsname {
    const SYSNAME: &str = "Linux";
    const NODENAME: &str = "Alien";
    const RELEASE: &str = "5.1";
    const VERSION: &str = "5.1";
    const MACHINE: &str = "riscv64";
    const DOMAINNAME: &str = "RustOS";
    let mut name = Utsname {
        sysname: [0; 65],
        nodename: [0; 65],
        release: [0; 65],
        version: [0; 65],
        machine: [0; 65],
        domainname: [0; 65],
    };
    name.sysname[..SYSNAME.len()].copy_from_slice(SYSNAME.as_bytes());
    name.nodename[..NODENAME.len()].copy_from_slice(NODENAME.as_bytes());
    name.release[..RELEASE.len()].copy_from_slice(RELEASE.as_bytes());
    name.version[..VERSION.len()].copy_from_slice(VERSION.as_bytes());
    name.machine[..MACHINE.len()].copy_from_slice(MACHINE.as_bytes());
    name.domainname[..DOMAINNAME.len()].copy_from_slice(DOMAINNAME.as_bytes());
    name
}

/// 一个系统调用，返回系统信息。信息包括操作系统名、在网络中的用户名、操作系统release和version版本、硬件类型、域名等信息，详情可见[`Utsname`]。
///
/// 函数成功执行后返回0。
#[syscall_func(160)]
pub fn uname(utsname: *const u8) -> isize {
    let task = current_task().unwrap();
    task.access_inner()
        .copy_to_user(&system_info(), utsname as *mut Utsname);
    0
}
