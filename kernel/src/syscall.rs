pub enum SysCallID{
    Read(usize,*const u8, usize),
    Write(usize,*mut u8, usize),
    Exit(i32),
    Fork,
    Yield,
    GetPID,
    GetTimeOfDay,
    ExecVe(*const u8),
    WaitPID(usize,*mut i32),
}

pub fn syscall(id:SysCallID)->isize{
    match id {
        SysCallID::Read(fd, buf, len) => {
            // sys_read(fd, buf, len)
            0
        }
        _ => -1
    }
}