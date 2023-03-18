use crate::sbi::shutdown;
use crate::{fs, task, timer};
#[macro_export]
macro_rules! syscall_id {
    ($name:ident,$val:expr) => {
        const $name: usize = $val;
    };
}

syscall_id!(SYSCALL_OPEN, 56);
syscall_id!(SYSCALL_CLOSE, 57);
syscall_id!(SYSCALL_LSEEK, 62);
syscall_id!(SYSCALL_MKDIR, 83);
syscall_id!(SYSCALL_RMDIR, 84);
syscall_id!(SYSCALL_UNLINK, 87);
syscall_id!(SYSCALL_GETCWD, 183);
syscall_id!(SYSCALL_READ, 63);
syscall_id!(SYSCALL_WRITE, 64);
syscall_id!(SYSCALL_EXIT, 93);
syscall_id!(SYSCALL_YIELD, 124);
syscall_id!(SYSCALL_GET_TIME, 169);
syscall_id!(SYSCALL_GETPID, 172);
syscall_id!(SYSCALL_FORK, 220);
syscall_id!(SYSCALL_EXEC, 221);
syscall_id!(SYSCALL_WAITPID, 260);
syscall_id!(SYSCALL_SHUTDOWN, 210);
syscall_id!(SYSCALL_LIST, 1000);
#[derive(Debug)]
pub enum SysCallID {
    Open(usize,u32),
    Close(usize),
    Read(usize, *mut u8, usize),
    Write(usize, *const u8, usize),
    Exit(i32),
    Fork,
    Yield,
    GetPID,
    GetTimeOfDay,
    ExecVe(*const u8,*const usize),
    WaitPID(isize, *mut i32),
    Shutdown,
    List,
    Unknown,
}

impl From<[usize; 4]> for SysCallID {
    fn from(value: [usize; 4]) -> Self {
        match value[0] {
            SYSCALL_OPEN => SysCallID::Open(value[1],value[2] as u32),
            SYSCALL_READ => SysCallID::Read(value[1], value[2] as *mut u8, value[3]),
            SYSCALL_WRITE => SysCallID::Write(value[1], value[2] as *const u8, value[3]),
            SYSCALL_EXIT => SysCallID::Exit(value[1] as i32),
            SYSCALL_FORK => SysCallID::Fork,
            SYSCALL_YIELD => SysCallID::Yield,
            SYSCALL_GET_TIME => SysCallID::GetTimeOfDay,
            SYSCALL_GETPID => SysCallID::GetPID,
            SYSCALL_EXEC => SysCallID::ExecVe(value[1] as *const u8,value[2] as *const usize),
            SYSCALL_WAITPID => SysCallID::WaitPID(value[1] as isize, value[2] as *mut i32),
            SYSCALL_SHUTDOWN => SysCallID::Shutdown,
            SYSCALL_LIST => SysCallID::List,
            SYSCALL_CLOSE => SysCallID::Close(value[1]),
            _ => SysCallID::Unknown,
        }
    }
}

pub trait Syscall {
    fn do_syscall(&self) -> isize;
}

impl Syscall for SysCallID {
    fn do_syscall(&self) -> isize {
        match *self {
            SysCallID::Open(path, flags) => fs::sys_open(path, flags),
            SysCallID::Close(fd) => fs::sys_close(fd),
            SysCallID::Read(fd, buf, len) => fs::sys_read(fd, buf, len),
            SysCallID::Write(fd, buf, len) => fs::sys_write(fd, buf, len),
            SysCallID::Exit(code) => task::do_exit(code),
            SysCallID::Fork => task::do_fork(),
            SysCallID::Yield => task::do_suspend(),
            SysCallID::GetPID => task::get_pid(),
            SysCallID::GetTimeOfDay => timer::get_time_ms() as isize,
            SysCallID::ExecVe(path,args) => task::do_exec(path,args),
            SysCallID::WaitPID(pid, status) => task::wait_pid(pid, status),
            SysCallID::Shutdown => shutdown(),
            SysCallID::List => fs::sys_list(),
            _ => -1,
        }
    }
}
