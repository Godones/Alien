use alloc::string::{String, ToString};

use numeric_enum_macro::numeric_enum;
use vfscore::error::VfsError;

numeric_enum!(
    #[repr(isize)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum LinuxErrno {
        EPERM = -1,
        ENOENT = -2,
        ESRCH = -3,
        EINTR = -4,
        EIO = -5,
        ENXIO = -6,
        E2BIG = -7,
        ENOEXEC = -8,
        EBADF = -9,
        ECHILD = -10,
        EAGAIN = -11,
        ENOMEM = -12,
        EACCES = -13,
        EFAULT = -14,
        ENOTBLK = -15,
        EBUSY = -16,
        EEXIST = -17,
        EXDEV = -18,
        ENODEV = -19,
        ENOTDIR = -20,
        EISDIR = -21,
        EINVAL = -22,
        ENFILE = -23,
        EMFILE = -24,
        ENOTTY = -25,
        ETXTBSY = -26,
        EFBIG = -27,
        ENOSPC = -28,
        ESPIPE = -29,
        EROFS = -30,
        EMLINK = -31,
        EPIPE = -32,
        EDOM = -33,
        ERANGE = -34,
        ENOSYS = -38,
        ELOOP = -40,
        EADDRINUSE = -98,
        /// 协议不被支持 Protocol not supported.
        EPROTONOSUPPORT = -92,
        EOPNOTSUPP = -94,
        EPFNOSUPPORT = -96,
        /// 不支持的地址
        EAFNOSUPPORT = -97,
        EADDRNOTAVAIL = -99,
        ENETDOWN = -100,
        ENETUNREACH = -101,
        ENETRESET = -102,
        ECONNABORTED = -103,
        ECONNRESET = -104,
        ENOBUFS = -105,
        EISCONN = -106,
        ENOTCONN = -107,
        /// 操作正在处理 Operation in progress.
        EINPROGRESS = -115,
        /// 拒绝连接
        ECONNREFUSED = -111,
        DOMAINCRASH = -255,
    }
);

impl ToString for LinuxErrno {
    fn to_string(&self) -> String {
        match self {
            LinuxErrno::EPERM => "Operation not permitted".to_string(),
            LinuxErrno::ENOENT => "No such file or directory".to_string(),
            LinuxErrno::ESRCH => "No such process".to_string(),
            LinuxErrno::EINTR => "Interrupted system call".to_string(),
            LinuxErrno::EIO => "I/O error".to_string(),
            LinuxErrno::ENXIO => "No such device or address".to_string(),
            LinuxErrno::E2BIG => "Argument list too long".to_string(),
            LinuxErrno::ENOEXEC => "Exec format error".to_string(),
            LinuxErrno::EBADF => "Bad file number".to_string(),
            LinuxErrno::ECHILD => "No child processes".to_string(),
            LinuxErrno::EAGAIN => "Try again".to_string(),
            LinuxErrno::ENOMEM => "Out of memory".to_string(),
            LinuxErrno::EACCES => "Permission denied".to_string(),
            LinuxErrno::EFAULT => "Bad address".to_string(),
            LinuxErrno::ENOTBLK => "Block device required".to_string(),
            LinuxErrno::EBUSY => "Device or resource busy".to_string(),
            LinuxErrno::EEXIST => "File exists".to_string(),
            LinuxErrno::EXDEV => "Cross-device link".to_string(),
            LinuxErrno::ENODEV => "No such device".to_string(),
            LinuxErrno::ENOTDIR => "Not a directory".to_string(),
            LinuxErrno::EISDIR => "Is a directory".to_string(),
            LinuxErrno::EINVAL => "Invalid argument".to_string(),
            LinuxErrno::ENFILE => "File table overflow".to_string(),
            LinuxErrno::EMFILE => "Too many open files".to_string(),
            LinuxErrno::ENOTTY => "Not a typewriter".to_string(),
            LinuxErrno::ETXTBSY => "Text file busy".to_string(),
            LinuxErrno::EFBIG => "File too large".to_string(),
            LinuxErrno::ENOSPC => "No space left on device".to_string(),
            LinuxErrno::ESPIPE => "Illegal seek".to_string(),
            LinuxErrno::EROFS => "Read-only file system".to_string(),
            LinuxErrno::EMLINK => "Too many links".to_string(),
            LinuxErrno::EPIPE => "Broken pipe".to_string(),
            LinuxErrno::EDOM => "Math argument out of domain of func".to_string(),
            LinuxErrno::ERANGE => "Math result not representable".to_string(),
            LinuxErrno::ENOSYS => "Function not implemented".to_string(),
            LinuxErrno::ELOOP => "Too many symbolic links encountered".to_string(),
            LinuxErrno::EADDRINUSE => "Address already in use".to_string(),
            LinuxErrno::EPROTONOSUPPORT => "Protocol not supported".to_string(),
            LinuxErrno::EOPNOTSUPP => "Operation not supported on transport endpoint".to_string(),
            LinuxErrno::EPFNOSUPPORT => "Protocol family not supported".to_string(),
            LinuxErrno::EAFNOSUPPORT => "Address family not supported by protocol".to_string(),
            LinuxErrno::EADDRNOTAVAIL => "Cannot assign requested address".to_string(),
            LinuxErrno::ENETDOWN => "Network is down".to_string(),
            LinuxErrno::ENETUNREACH => "Network is unreachable".to_string(),
            LinuxErrno::ENETRESET => "Network dropped connection because of reset".to_string(),
            LinuxErrno::ECONNABORTED => "Software caused connection abort".to_string(),
            LinuxErrno::ECONNRESET => "Connection reset by peer".to_string(),
            LinuxErrno::ENOBUFS => "No buffer space available".to_string(),
            LinuxErrno::EISCONN => "Transport endpoint is already connected".to_string(),
            LinuxErrno::ENOTCONN => "Transport endpoint is not connected".to_string(),
            LinuxErrno::EINPROGRESS => "Connection already in progress".to_string(),
            LinuxErrno::ECONNREFUSED => "Connection refused".to_string(),
            LinuxErrno::DOMAINCRASH => "Domain crash".to_string(),
            _ => "Unknown error".to_string(),
        }
    }
}

#[cfg(feature = "trick")]
impl syscall_table::ToIsize for LinuxErrno {
    fn to_isize(self) -> isize {
        self as isize
    }
}

impl From<VfsError> for LinuxErrno {
    fn from(value: VfsError) -> Self {
        LinuxErrno::try_from(-(i32::from(value) as isize)).unwrap_or(LinuxErrno::EINVAL)
    }
}

impl From<LinuxErrno> for VfsError {
    fn from(value: LinuxErrno) -> Self {
        let code = -(value as i32);
        VfsError::from(code)
    }
}

const SYSCALL_GETCWD: usize = 17;
const SYSCALL_DUP: usize = 23;
const SYSCALL_DUP3: usize = 24;
const SYSCALL_FCNTL: usize = 25;
const SYSCALL_IOCTL: usize = 29;
const SYSCALL_MKNODAT: usize = 33;
const SYSCALL_MKDIRAT: usize = 34;
const SYSCALL_UNLINKAT: usize = 35;
const SYSCALL_LINKAT: usize = 37;
const SYSCALL_UMOUNT2: usize = 39;
const SYSCALL_MOUNT: usize = 40;
const SYSCALL_STATFS: usize = 43;
const SYSCALL_FTRUNCATE: usize = 46;
const SYSCALL_FACCESSAT: usize = 48;
const SYSCALL_CHDIR: usize = 49;
const SYSCALL_FCHMOD: usize = 52;
const SYSCALL_FCHMODAT: usize = 53;
const SYSCALL_FCHOWN: usize = 55;
const SYSCALL_OPENAT: usize = 56;
const SYSCALL_CLOSE: usize = 57;
const SYSCALL_PIPE2: usize = 59;
const SYSCALL_GETDENTS64: usize = 61;
const SYSCALL_LSEEK: usize = 62;
const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_READV: usize = 65;
const SYSCALL_WRITEV: usize = 66;
const SYSCALL_PREAD: usize = 67;
const SYSCALL_PWRITE: usize = 68;
const SYSCALL_SENDFILE: usize = 71;
const SYSCALL_PSELECT6: usize = 72;
const SYSCALL_PPOLL: usize = 73;
const SYSCALL_READLINKAT: usize = 78;
const SYSCALL_FSTATAT: usize = 79;
const SYSCALL_FSTAT: usize = 80;
const SYSCALL_SYNC: usize = 81;
const SYSCALL_FSYNC: usize = 82;
const SYSCALL_UTIMENSAT: usize = 88;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_EXIT_GROUP: usize = 94;
const SYSCALL_SET_TID_ADDRESS: usize = 96;
const SYSCALL_FUTEX: usize = 98;
const SYSCALL_SET_ROBUST_LIST: usize = 99;
const SYSCALL_GET_ROBUST_LIST: usize = 100;
const SYSCALL_NANOSLEEP: usize = 101;
const SYSCALL_GETITIMER: usize = 102;
const SYSCALL_SETITIMER: usize = 103;
const SYSCALL_CLOCK_GETTIME: usize = 113;

const SYSCALL_CLOCK_GETRES: usize = 114;
const SYSCALL_CLOCK_NANOSLEEP: usize = 115;
const SYSCALL_SYSLOG: usize = 116;
const SYSCALL_YIELD: usize = 124;

const SYSCALL_SCHED_SETPARAM: usize = 118;
const SYSCALL_SCHED_GETPARAM: usize = 121;
const SYSCALL_SCHED_SETAFFINITY: usize = 122;
const SYSCALL_SCHED_GETAFFINITY: usize = 123;
const SYSCALL_SCHED_GETSCHEDULER: usize = 120;
const SYSCALL_SCHED_SETSCHEDULER: usize = 119;

const SYSCALL_KILL: usize = 129;
const SYSCALL_TKILL: usize = 130;
const SYSCALL_SIGSUSPEND: usize = 133;

const SYSCALL_SIGACTION: usize = 134;
const SYSCALL_SIGPROCMASK: usize = 135;
const SYSCALL_SIGTIMEDWAIT: usize = 137;
const SYSCALL_SIGRETURN: usize = 139;
const SYSCALL_TIMES: usize = 153;
const SYSCALL_SETPGID: usize = 154;
const SYSCALL_GETPGID: usize = 155;
const SYSCALL_SETSID: usize = 157;
const SYSCALL_UNAME: usize = 160;
const SYSCALL_GETRUSAGE: usize = 165;
const SYSCALL_UMASK: usize = 166;
const SYSCALL_GET_TIME_OF_DAY: usize = 169;
const SYSCALL_GETPID: usize = 172;
const SYSCALL_GETPPID: usize = 173;
const SYSCALL_GETUID: usize = 174;
const SYSCALL_GETEUID: usize = 175;
const SYSCALL_GETGID: usize = 176;
const SYSCALL_GETEGID: usize = 177;
const SYSCALL_GETTID: usize = 178;
const SYSCALL_SYSINFO: usize = 179;
const SYSCALL_SHMGET: usize = 194;
const SYSCALL_SHAMCTL: usize = 195;
const SYSCALL_SHAMAT: usize = 196;
const SYSCALL_SHAMDT: usize = 197;
const SYSCALL_SOCKET: usize = 198;
const SYSCALL_BIND: usize = 200;
const SYSCALL_LISTEN: usize = 201;
const SYSCALL_ACCEPT: usize = 202;
const SYSCALL_CONNECT: usize = 203;
const SYSCALL_GETSOCKNAME: usize = 204;
const SYSCALL_GETPEERNAME: usize = 205;
const SYSCALL_SENDTO: usize = 206;
const SYSCALL_RECVFROM: usize = 207;
const SYSCALL_SETSOCKOPT: usize = 208;
const SYSCALL_GETSOCKOPT: usize = 209;
const SYSCALL_SBRK: usize = 213;
const SYSCALL_BRK: usize = 214;
const SYSCALL_MUNMAP: usize = 215;
// Warning, we don't implement clone, we implement fork instead.
const SYSCALL_CLONE: usize = 220;
// fork is implemented as clone(SIGCHLD, 0) in lib.
const SYSCALL_EXECVE: usize = 221;
const SYSCALL_MMAP: usize = 222;
const SYSCALL_MPROTECT: usize = 226;
const SYSCALL_MSYNC: usize = 227;

const SYSCALL_MADVISE: usize = 233;
const SYSCALL_WAIT4: usize = 260;
// wait is implemented as wait4(pid, status, options, 0) in lib.
const SYSCALL_PRLIMIT: usize = 261;
const SYSCALL_RENAMEAT2: usize = 276;
const SYSCALL_MEMBARRIER: usize = 283;
const SYSCALL_FACCESSAT2: usize = 439;
const SYSCALL_SHUTDOWN: usize = 210;
// Not standard POSIX sys_call
const SYSCALL_COPY_FILE_RANGE: usize = 285;
const SYSCALL_LS: usize = 500;
const SYSCALL_CLEAR: usize = 502;
const SYSCALL_OPEN: usize = 506;
//where?
const SYSCALL_GET_TIME: usize = 1690; //you mean get time of day by 169?

pub fn syscall_name(id: usize) -> &'static str {
    match id {
        SYSCALL_COPY_FILE_RANGE => "copy_file_range",
        SYSCALL_FCHOWN => "fchown",
        SYSCALL_SETSID => "setsid",
        SYSCALL_SIGSUSPEND => "sigsuspend",
        SYSCALL_MADVISE => "madvise",
        SYSCALL_CLOCK_NANOSLEEP => "clock_nanosleep",
        SYSCALL_CLOCK_GETRES => "clock_getres",
        SYSCALL_SCHED_SETPARAM => "sched_setparam",
        SYSCALL_SCHED_GETPARAM => "sched_getparam",
        SYSCALL_SCHED_SETAFFINITY => "sched_setaffinity",
        SYSCALL_SCHED_GETAFFINITY => "sched_getaffinity",
        SYSCALL_SCHED_GETSCHEDULER => "sched_getscheduler",
        SYSCALL_SCHED_SETSCHEDULER => "sched_setscheduler",
        SYSCALL_SHUTDOWN => "shutdown",
        SYSCALL_FCHMODAT => "fchmodat",
        SYSCALL_FCHMOD => "fchmod",
        SYSCALL_GETSOCKOPT => "getsockopt",
        SYSCALL_MKNODAT => "mknodat",
        SYSCALL_SHMGET => "shmget",
        SYSCALL_SHAMCTL => "shamctl",
        SYSCALL_SHAMAT => "shamat",
        SYSCALL_SHAMDT => "shamdt",
        SYSCALL_DUP => "dup",
        SYSCALL_DUP3 => "dup3",
        SYSCALL_OPEN => "open",
        SYSCALL_GET_TIME => "get_time",
        SYSCALL_GETCWD => "getcwd",
        SYSCALL_FCNTL => "fcntl",
        SYSCALL_IOCTL => "ioctl",
        SYSCALL_MKDIRAT => "mkdirat",
        SYSCALL_UNLINKAT => "unlinkat",
        SYSCALL_LINKAT => "linkat",
        SYSCALL_UMOUNT2 => "umount2",
        SYSCALL_MOUNT => "mount",
        SYSCALL_FACCESSAT => "faccessat",
        SYSCALL_CHDIR => "chdir",
        SYSCALL_OPENAT => "openat",
        SYSCALL_CLOSE => "close",
        SYSCALL_PIPE2 => "pipe2",
        SYSCALL_GETDENTS64 => "getdents64",
        SYSCALL_LSEEK => "lseek",
        SYSCALL_READ => "read",
        SYSCALL_WRITE => "write",
        SYSCALL_READV => "readv",
        SYSCALL_WRITEV => "writev",
        SYSCALL_PREAD => "pread",
        SYSCALL_PWRITE => "pwrite",
        SYSCALL_SENDFILE => "sendfile",
        SYSCALL_PSELECT6 => "pselect6",
        SYSCALL_PPOLL => "ppoll",
        SYSCALL_READLINKAT => "readlinkat",
        SYSCALL_FSTATAT => "fstatat",
        SYSCALL_FSTAT => "fstat",
        SYSCALL_STATFS => "statfs",
        SYSCALL_FTRUNCATE => "ftruncate",
        SYSCALL_SYNC => "sync",
        SYSCALL_FSYNC => "fsync",
        SYSCALL_UTIMENSAT => "utimensat",
        SYSCALL_EXIT => "exit",
        SYSCALL_EXIT_GROUP => "exit_GROUP",
        SYSCALL_SET_TID_ADDRESS => "set_tid_address",
        SYSCALL_FUTEX => "futex",
        SYSCALL_SET_ROBUST_LIST => "set_robust_list",
        SYSCALL_GET_ROBUST_LIST => "get_robust_list",
        SYSCALL_NANOSLEEP => "nanosleep",
        SYSCALL_GETITIMER => "getitimer",
        SYSCALL_SETITIMER => "setitimer",
        SYSCALL_CLOCK_GETTIME => "clock_gettime",
        SYSCALL_SYSLOG => "syslog",
        SYSCALL_YIELD => "yield",
        SYSCALL_KILL => "kill",
        SYSCALL_TKILL => "tkill",
        SYSCALL_SIGACTION => "sigaction",
        SYSCALL_SIGPROCMASK => "sigprocmask",
        SYSCALL_SIGTIMEDWAIT => "sigtimedwait",
        SYSCALL_SIGRETURN => "sigreturn",
        SYSCALL_TIMES => "times",
        SYSCALL_SETPGID => "setpgid",
        SYSCALL_GETPGID => "getpgid",
        SYSCALL_UNAME => "uname",
        SYSCALL_GETRUSAGE => "getrusage",
        SYSCALL_UMASK => "umask",
        SYSCALL_GET_TIME_OF_DAY => "get_time_of_day",
        SYSCALL_GETPID => "getpid",
        SYSCALL_GETPPID => "getppid",
        SYSCALL_GETUID => "getuid",
        SYSCALL_GETEUID => "geteuid",
        SYSCALL_GETGID => "getgid",
        SYSCALL_GETEGID => "getegid",
        SYSCALL_GETTID => "gettid",
        SYSCALL_SYSINFO => "sysinfo",
        SYSCALL_SOCKET => "socket",
        SYSCALL_BIND => "bind",
        SYSCALL_LISTEN => "listen",
        SYSCALL_ACCEPT => "accept",
        SYSCALL_CONNECT => "connect",
        SYSCALL_GETSOCKNAME => "getsockname",
        SYSCALL_GETPEERNAME => "getpeername",
        SYSCALL_SENDTO => "sendto",
        SYSCALL_RECVFROM => "recvfrom",
        SYSCALL_SETSOCKOPT => "setsockopt",
        SYSCALL_SBRK => "sbrk",
        SYSCALL_BRK => "brk",
        SYSCALL_MUNMAP => "munmap",
        SYSCALL_CLONE => "clone",
        SYSCALL_EXECVE => "execve",
        SYSCALL_MMAP => "mmap",
        SYSCALL_MPROTECT => "mprotect",
        SYSCALL_MSYNC => "msync",
        SYSCALL_WAIT4 => "wait4",
        SYSCALL_PRLIMIT => "prlimit",
        SYSCALL_RENAMEAT2 => "renameat2",
        SYSCALL_FACCESSAT2 => "faccessat2",
        SYSCALL_MEMBARRIER => "membarrier",
        // non-standard
        SYSCALL_LS => "ls",
        SYSCALL_SHUTDOWN => "shutdown",
        SYSCALL_CLEAR => "clear",
        _ => "unknown",
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PrLimit {
    /// 软上限
    pub rlim_cur: u64,
    /// 硬上限
    pub rlim_max: u64,
}

impl PrLimit {
    pub fn new(cur: u64, max: u64) -> Self {
        Self {
            rlim_cur: cur,
            rlim_max: max,
        }
    }
}

/// 用户栈大小
// pub const RLIMIT_STACK: i32 = 3;
// 可以打开的 fd 数
// pub const RLIMIT_NOFILE: i32 = 7;
// 用户地址空间的最大大小
// pub const RLIMIT_AS: i32 = 9;
numeric_enum_macro::numeric_enum! {
    #[repr(usize)]
    #[derive(Eq, PartialEq, Debug, Copy, Clone)]
    pub enum PrLimitRes{
        RlimitStack = 3,
        RlimitNofile = 7,
        RlimitAs = 9
    }
}
