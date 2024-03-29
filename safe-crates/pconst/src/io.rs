use bitflags::bitflags;

#[derive(Debug, Clone, Copy)]
pub struct IoVec {
    pub base: *mut u8,
    pub len: usize,
}

impl IoVec {
    pub fn new(base: *mut u8, len: usize) -> Self {
        Self { base, len }
    }
    pub fn empty() -> Self {
        Self {
            base: core::ptr::null_mut(),
            len: 0,
        }
    }
}

numeric_enum_macro::numeric_enum! {
    #[repr(usize)]
    #[allow(non_camel_case_types)]
    #[derive(Debug)]
    /// sys_fcntl64 使用的选项
    pub enum Fcntl64Cmd {
        /// 复制这个 fd，相当于 sys_dup
        F_DUPFD = 0,
        /// 获取 cloexec 信息，即 exec 成功时是否删除该 fd
        F_GETFD = 1,
        /// 设置 cloexec 信息，即 exec 成功时删除该 fd
        F_SETFD = 2,
        /// 获取 flags 信息
        F_GETFL = 3,
        /// 设置 flags 信息
        F_SETFL = 4,
        /// 复制 fd，然后设置 cloexec 信息，即 exec 成功时删除该 fd
        GETLK = 5,
        SETLK = 6,
        SETLKW = 7,
        SETOWN = 8,
        GETOWN = 9,
        SETSIG = 10,
        GETSIG = 11,
        SETOWN_EX = 15,
        GETOWN_EX = 16,
        GETOWNER_UIDS = 17,
        OFD_GETLK = 36,
        OFD_SETLK = 37,
        OFD_SETLKW = 38,
        SETLEASE = 1024,
        GETLEASE = 1025,
        NOTIFY = 1026,
        CANCELLK = 1029,
        F_DUPFD_CLOEXEC = 1030,
        SETPIPE_SZ = 1031,
        GETPIPE_SZ = 1032,
        ADD_SEALS = 1033,
        GET_SEALS = 1034,
        GET_RW_HINT = 1035,
        SET_RW_HINT = 1036,
        GET_FILE_RW_HINT = 1037,
        SET_FILE_RW_HINT = 1038,
        Unknown = 0xffff,
    }
}

bitflags! {
    pub struct MapFlags: u32 {
        /// 该段映射为共享映射，多个进程对该文件映射共享
        const MAP_SHARED = 0x01;
        /// (未使用到)对映射区域的写入操作会产生一个映射文件的复制，即私人的"写入时复制"(copy on write)对此区域作的任何修改都不会写回原来的文件内容。
        const MAP_PRIVATE = 0x02;
        /// 直接采用`addr`作为绝对地址创建映射。如果参数`start`所指的地址无法成功建立映射时，则放弃映射，不对地址做修正。
        const MAP_FIXED = 0x10;
        /// 不采用文件映射。
        const MAP_ANONYMOUS = 0x20;
        /// 映射时不保留空间，即可能在实际使用mmp出来的内存时内存溢出
        const MAP_NORESERVE = 1 << 14;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FsStat {
    /// 是个 magic number，每个知名的 fs 都各有定义，但显然我们没有
    pub f_type: i64,
    /// 最优传输块大小
    pub f_bsize: i64,
    /// 总的块数
    pub f_blocks: u64,
    /// 还剩多少块未分配
    pub f_bfree: u64,
    /// 对用户来说，还有多少块可用
    pub f_bavail: u64,
    /// 总的 inode 数
    pub f_files: u64,
    /// 空闲的 inode 数
    pub f_ffree: u64,
    /// 文件系统编号，但实际上对于不同的OS差异很大，所以不会特地去用
    pub f_fsid: [i32; 2],
    /// 文件名长度限制，这个OS默认FAT已经使用了加长命名
    pub f_namelen: isize,
    /// 片大小
    pub f_frsize: isize,
    /// 一些选项，但其实也没用到
    pub f_flags: isize,
    /// 空余 padding
    pub f_spare: [isize; 4],
}

impl Default for FsStat {
    fn default() -> Self {
        Self {
            f_type: 0,
            f_bsize: 0,
            f_blocks: 0,
            f_bfree: 0,
            f_bavail: 0,
            f_files: 0,
            f_ffree: 0,
            f_fsid: [0, 0],
            f_namelen: 0,
            f_frsize: 0,
            f_flags: 0,
            f_spare: [0; 4],
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct FileStat {
    pub st_dev: u64,
    pub st_ino: u64,
    pub st_mode: u32,
    pub st_nlink: u32,
    pub st_uid: u32,
    pub st_gid: u32,
    pub st_rdev: u64,
    __pad: u64,
    pub st_size: u64,
    pub st_blksize: u32,
    __pad2: u32,
    pub st_blocks: u64,
    pub st_atime_sec: u64,
    pub st_atime_nsec: u64,
    pub st_mtime_sec: u64,
    pub st_mtime_nsec: u64,
    pub st_ctime_sec: u64,
    pub st_ctime_nsec: u64,
    unused: u64,
} //128
bitflags! {
    /// ppoll 使用，表示对应在文件上等待或者发生过的事件
    pub struct PollEvents: u16 {
        /// 可读
        const IN = 0x0001;
        /// 可写
        const OUT = 0x0004;
        /// 报错
        const ERR = 0x0008;
        /// 已终止，如 pipe 的另一端已关闭连接的情况
        const HUP = 0x0010;
        /// 无效的 fd
        const INVAL = 0x0020;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
/// ppoll 系统调用参数用到的结构
pub struct PollFd {
    /// 等待的 fd
    pub fd: i32,
    /// 等待的事件
    pub events: PollEvents,
    /// 返回的事件
    pub revents: PollEvents,
}

bitflags! {
    pub struct FaccessatMode: u32 {
        const F_OK = 0;
        const R_OK = 4;
        const W_OK = 2;
        const X_OK = 1;
    }
    pub struct FaccessatFlags: u32 {
        const AT_SYMLINK_NOFOLLOW = 0x100;
        const AT_EACCESS = 0x200;
    }
}

bitflags! {
    pub struct UnlinkatFlags: u32 {
        const AT_REMOVEDIR = 0x200;
    }
}

bitflags! {
    pub struct LinkFlags:u32{
        /// Follow symbolic links.
        const AT_SYMLINK_FOLLOW = 0x400;
        /// Allow empty relative pathname.
        const AT_EMPTY_PATH = 0x1000;
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct WinSize {
    ws_row: u16,
    ws_col: u16,
    xpixel: u16,
    ypixel: u16,
}

impl Default for WinSize {
    fn default() -> Self {
        Self {
            ws_row: 24,
            ws_col: 80,
            xpixel: 0,
            ypixel: 0,
        }
    }
}

numeric_enum_macro::numeric_enum! {
    #[repr(u32)]
    #[allow(non_camel_case_types)]
    #[derive(Debug, Eq, PartialEq,Copy, Clone)]
    pub enum TeletypeCommand {
        // For struct termios
        /// Gets the current serial port settings.
        TCGETS = 0x5401,
        /// Sets the serial port settings immediately.
        TCSETS = 0x5402,
        /// Sets the serial port settings after allowing the input and output buffers to drain/empty.
        TCSETSW = 0x5403,
        /// Sets the serial port settings after flushing the input and output buffers.
        TCSETSF = 0x5404,
        /// For struct termio
        /// Gets the current serial port settings.
        TCGETA = 0x5405,
        /// Sets the serial port settings immediately.
        TCSETA = 0x5406,
        /// Sets the serial port settings after allowing the input and output buffers to drain/empty.
        TCSETAW = 0x5407,
        /// Sets the serial port settings after flushing the input and output buffers.
        TCSETAF = 0x5408,
        /// Get the process group ID of the foreground process group on this terminal.
        TIOCGPGRP = 0x540F,
        /// Set the foreground process group ID of this terminal.
        TIOCSPGRP = 0x5410,
        /// Get window size.
        TIOCGWINSZ = 0x5413,
        /// Set window size.
        TIOCSWINSZ = 0x5414,
        /// Non-cloexec
        FIONCLEX = 0x5450,
        /// Cloexec
        FIOCLEX = 0x5451,
        /// rustc using pipe and ioctl pipe file with this request id
        /// for non-blocking/blocking IO control setting
        FIONBIO = 0x5421,
        /// Read time
        RTC_RD_TIME = 0x80247009,
        ILLEAGAL = 0,
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// The termios functions describe a general terminal interface that
/// is provided to control asynchronous communications ports.
pub struct Termios {
    /// input modes
    pub iflag: u32,
    /// ouput modes
    pub oflag: u32,
    /// control modes
    pub cflag: u32,
    /// local modes
    pub lflag: u32,
    pub line: u8,
    /// terminal special characters.
    pub cc: [u8; 32],
    pub ispeed: u32,
    pub ospeed: u32,
}

impl Default for Termios {
    fn default() -> Self {
        Termios {
            // IMAXBEL | IUTF8 | IXON | IXANY | ICRNL | BRKINT
            iflag: 0o66402,
            // OPOST | ONLCR
            oflag: 0o5,
            // HUPCL | CREAD | CSIZE | EXTB
            cflag: 0o2277,
            // IEXTEN | ECHOTCL | ECHOKE ECHO | ECHOE | ECHOK | ISIG | ICANON
            lflag: 0o105073,
            line: 0,
            cc: [
                3,   // VINTR Ctrl-C
                28,  // VQUIT
                127, // VERASE
                21,  // VKILL
                4,   // VEOF Ctrl-D
                0,   // VTIME
                1,   // VMIN
                0,   // VSWTC
                17,  // VSTART
                19,  // VSTOP
                26,  // VSUSP Ctrl-Z
                255, // VEOL
                18,  // VREPAINT
                15,  // VDISCARD
                23,  // VWERASE
                22,  // VLNEXT
                255, // VEOL2
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
            ispeed: 0,
            ospeed: 0,
        }
    }
}

bitflags! {
    pub struct LocalModes : u32 {
        const ISIG = 0o000001;
        const ICANON = 0o000002;
        const ECHO = 0o000010;
        const ECHOE = 0o000020;
        const ECHOK = 0o000040;
        const ECHONL = 0o000100;
        const NOFLSH = 0o000200;
        const TOSTOP = 0o000400;
        const IEXTEN = 0o100000;
        const XCASE = 0o000004;
        const ECHOCTL = 0o001000;
        const ECHOPRT = 0o002000;
        const ECHOKE = 0o004000;
        const FLUSHO = 0o010000;
        const PENDIN = 0o040000;
        const EXTPROC = 0o200000;
    }
}

bitflags::bitflags! {
    pub struct OpenFlags: usize {
        // reserve 3 bits for the access mode
        const O_RDONLY      = 0;
        const O_WRONLY      = 1;
        const O_RDWR        = 2;
        const O_ACCMODE     = 3;
        const O_CREAT       = 0o100;
        const O_EXCL        = 0o200;
        const O_NOCTTY      = 0o400;
        const O_TRUNC       = 0o1000;
        const O_APPEND      = 0o2000;
        const O_NONBLOCK    = 0o4000;
        const O_DSYNC       = 0o10000;
        const O_SYNC        = 0o4010000;
        const O_RSYNC       = 0o4010000;
        const O_DIRECTORY   = 0o200000;
        const O_NOFOLLOW    = 0o400000;
        const O_CLOEXEC     = 0o2000000;

        const O_ASYNC       = 0o20000;
        const O_DIRECT      = 0o40000;
        const O_LARGEFILE   = 0o100000;
        const O_NOATIME     = 0o1000000;
        const O_PATH        = 0o10000000;
        const O_TMPFILE     = 0o20200000;
    }
}
/// Enumeration of possible methods to seek within an I/O object.
///
/// It is used by the [`Seek`] trait.
#[derive(Copy, PartialEq, Eq, Clone, Debug)]
pub enum SeekFrom {
    /// Sets the offset to the provided number of bytes.
    Start(u64),
    /// Sets the offset to the size of this object plus the specified number of
    /// bytes.
    ///
    /// It is possible to seek beyond the end of an object, but it's an error to
    /// seek before byte 0.
    End(i64),
    /// Sets the offset to the current position plus the specified number of
    /// bytes.
    ///
    /// It is possible to seek beyond the end of an object, but it's an error to
    /// seek before byte 0.
    Current(i64),
}

impl TryFrom<(usize, usize)> for SeekFrom {
    type Error = ();

    fn try_from(value: (usize, usize)) -> Result<Self, Self::Error> {
        match value {
            (0, offset) => Ok(SeekFrom::Start(offset as u64)),
            (1, offset) => Ok(SeekFrom::Current(offset as i64)),
            (2, offset) => Ok(SeekFrom::End(offset as i64)),
            _ => Err(()),
        }
    }
}

#[repr(C)]
pub struct Dirent64 {
    /// ino is an inode number
    pub ino: u64,
    /// off is an offset to next linux_dirent
    pub off: i64,
    /// reclen is the length of this linux_dirent
    pub reclen: u16,
    /// type is the file type
    pub type_: DirentType,
    /// name is the filename (null-terminated)
    pub name: [u8; 0],
}

impl Dirent64 {
    pub fn new(name: &str, ino: u64, off: i64, type_: DirentType) -> Self {
        let size = core::mem::size_of::<Self>() + name.len() + 1;
        // align to 8 bytes
        let size = (size + 7) & !7;
        Self {
            ino,
            off,
            reclen: size as u16,
            type_,
            name: [0; 0],
        }
    }
    pub unsafe fn get_name(&self) -> &str {
        let name = self.name.as_ptr();
        let name = core::ffi::CStr::from_ptr(name as *const i8);
        name.to_str().unwrap()
    }
    pub fn len(&self) -> usize {
        self.reclen as usize
    }
}
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum DirentType {
    DT_UNKNOWN = 0,
    DT_FIFO = 1,
    DT_CHR = 2,
    DT_DIR = 4,
    DT_BLK = 6,
    DT_REG = 8,
    DT_LNK = 10,
    DT_SOCK = 12,
    DT_WHT = 14,
}

impl DirentType {
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::DT_FIFO,
            2 => Self::DT_CHR,
            4 => Self::DT_DIR,
            6 => Self::DT_BLK,
            8 => Self::DT_REG,
            10 => Self::DT_LNK,
            12 => Self::DT_SOCK,
            14 => Self::DT_WHT,
            _ => Self::DT_UNKNOWN,
        }
    }
}

bitflags! {
    pub struct MountFlags:u32{
        /// This filesystem is mounted read-only.
        const MS_RDONLY = 1;
        /// The set-user-ID and set-group-ID bits are ignored by exec(3) for executable files on this filesystem.
        const MS_NOSUID = 1 << 1;
        /// Disallow access to device special files on this filesystem.
        const MS_NODEV = 1 << 2;
        /// Execution of programs is disallowed on this filesystem.
        const MS_NOEXEC = 1 << 3;
        /// Writes are synched to the filesystem immediately (see the description of O_SYNC in open(2)).
        const MS_SYNCHRONOUS = 1 << 4;
        /// Alter flags of a mounted FS
        const MS_REMOUNT = 1 << 5;
        /// Allow mandatory locks on an FS
        const MS_MANDLOCK = 1 << 6;
        /// Directory modifications are synchronous
        const MS_DIRSYNC = 1 << 7;
        /// Do not follow symlinks
        const MS_NOSYMFOLLOW = 1 << 8;
        /// Do not update access times.
        const MS_NOATIME = 1 << 10;
        /// Do not update directory access times
        const MS_NODEIRATIME = 1 << 11;
        const MS_BIND = 1 << 12;
        const MS_MOVE = 1 << 13;
        const MS_REC = 1 << 14;
        /// War is peace. Verbosity is silence.
        const MS_SILENT = 1 << 15;
        /// VFS does not apply the umask
        const MS_POSIXACL = 1 << 16;
        /// change to unbindable
        const MS_UNBINDABLE = 1 << 17;
        /// change to private
        const MS_PRIVATE = 1 << 18;
        /// change to slave
        const MS_SLAVE = 1 << 19;
        /// change to shared
        const MS_SHARED = 1 << 20;
        /// Update atime relative to mtime/ctime.
        const MS_RELATIME = 1 << 21;
        /// this is a kern_mount call
        const MS_KERNMOUNT = 1 << 22;
        /// Update inode I_version field
        const MS_I_VERSION = 1 << 23;
        /// Always perform atime updates
        const MS_STRICTATIME = 1 << 24;
        /// Update the on-disk [acm]times lazily
        const MS_LAZYTIME = 1 << 25;
        /// These sb flags are internal to the kernel
        const MS_SUBMOUNT = 1 << 26;
        const MS_NOREMOTELOCK = 1 << 27;
        const MS_NOSEC = 1 << 28;
        const MS_BORN = 1 << 29;
        const MS_ACTIVE = 1 << 30;
        const MS_NOUSER = 1 << 31;
    }
}

bitflags! {
    pub struct InodeMode: u32 {
        /// Type
        const TYPE_MASK = 0o170000;
        /// FIFO
        const FIFO  = 0o010000;
        /// character device
        const CHAR  = 0o020000;
        /// directory
        const DIR   = 0o040000;
        /// block device
        const BLOCK = 0o060000;
        /// ordinary regular file
        const FILE  = 0o100000;
        /// symbolic link
        const LINK  = 0o120000;
        /// socket
        const SOCKET = 0o140000;

        /// Set-user-ID on execution.
        const SET_UID = 0o4000;
        /// Set-group-ID on execution.
        const SET_GID = 0o2000;
        /// sticky bit
        const STICKY = 0o1000;
        /// Read, write, execute/search by owner.
        const OWNER_MASK = 0o700;
        /// Read permission, owner.
        const OWNER_READ = 0o400;
        /// Write permission, owner.
        const OWNER_WRITE = 0o200;
        /// Execute/search permission, owner.
        const OWNER_EXEC = 0o100;

        /// Read, write, execute/search by group.
        const GROUP_MASK = 0o70;
        /// Read permission, group.
        const GROUP_READ = 0o40;
        /// Write permission, group.
        const GROUP_WRITE = 0o20;
        /// Execute/search permission, group.
        const GROUP_EXEC = 0o10;

        /// Read, write, execute/search by others.
        const OTHER_MASK = 0o7;
        /// Read permission, others.
        const OTHER_READ = 0o4;
        /// Write permission, others.
        const OTHER_WRITE = 0o2;
        /// Execute/search permission, others.
        const OTHER_EXEC = 0o1;
    }
}

bitflags! {
    pub struct StatFlags:u32{
        const AT_EMPTY_PATH = 0x1000;
        const AT_NO_AUTOMOUNT = 0x800;
        const AT_SYMLINK_NOFOLLOW = 0x100;
    }
}

bitflags! {
     /// renameat flag
    pub struct Renameat2Flags: u32 {
        /// Go back to renameat
        const RENAME_NONE = 0;
        /// Atomically exchange oldpath and newpath.
        const RENAME_EXCHANGE = 1 << 1;
        /// Don't overwrite newpath of the rename. Return an error if newpath already exists.
        const RENAME_NOREPLACE = 1 << 0;
        /// This operation makes sense only for overlay/union filesystem implementations.
        const RENAME_WHITEOUT = 1 << 2;
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
pub struct RtcTime {
    pub sec: u32,
    pub min: u32,
    pub hour: u32,
    pub mday: u32,
    pub mon: u32,
    pub year: u32,
    pub wday: u32,  // unused
    pub yday: u32,  // unused
    pub isdst: u32, // unused
}
