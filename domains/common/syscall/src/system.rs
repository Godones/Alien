use alloc::sync::Arc;

use basic::AlienResult;
use interface::TaskDomain;
use pod::Pod;

pub fn sys_uname(task_domain: &Arc<dyn TaskDomain>, utsname: usize) -> AlienResult<isize> {
    let info = system_info();
    task_domain.copy_to_user(utsname, info.as_bytes())?;
    Ok(0)
}
#[repr(C)]
#[derive(Copy, Clone, Pod)]
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
