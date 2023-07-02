use syscall_table::syscall_func;

use crate::task::current_task;

#[repr(C)]
pub struct Utsname {
    sysname: [u8; 65],
    nodename: [u8; 65],
    release: [u8; 65],
    version: [u8; 65],
    machine: [u8; 65],
    domainname: [u8; 65],
}

fn system_info() -> Utsname {
    const SYSNAME: &str = "RustOS";
    const NODENAME: &str = "RustOS";
    const RELEASE: &str = "0.1";
    const VERSION: &str = "0.1";
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

#[syscall_func(160)]
pub fn sys_uname(utsname: *const u8) -> isize {
    let process = current_task().unwrap();
    let utsname = process.transfer_raw_ptr(utsname as *mut Utsname);
    *utsname = system_info();
    0
}
