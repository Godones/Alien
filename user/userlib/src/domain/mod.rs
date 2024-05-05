use crate::syscall::{sys_register_domain, sys_update_domain};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainTypeRaw {
    FsDomain = 1,
    BlkDeviceDomain = 2,
    CacheBlkDeviceDomain = 3,
    RtcDomain = 4,
    GpuDomain = 5,
    InputDomain = 6,
    VfsDomain = 7,
    UartDomain = 8,
    PLICDomain = 9,
    TaskDomain = 10,
    SysCallDomain = 11,
    ShadowBlockDomain = 12,
    BufUartDomain = 13,
    NetDeviceDomain = 14,
    BufInputDomain = 15,
    EmptyDeviceDomain = 16,
    DevFsDomain = 17,
    SchedulerDomain = 18,
    LogDomain = 19,
}
pub fn register_domain(fd: usize, ty: DomainTypeRaw, name: &str) -> isize {
    sys_register_domain(fd, ty as u8, name.as_ptr(), name.len())
}

pub fn update_domain(old_name: &str, new_name: &str, ty: DomainTypeRaw) -> isize {
    sys_update_domain(
        old_name.as_ptr(),
        old_name.len(),
        new_name.as_ptr(),
        new_name.len(),
        ty as u8,
    )
}
