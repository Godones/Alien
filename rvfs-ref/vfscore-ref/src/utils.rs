use alloc::string::String;

use bitflags::bitflags;
#[cfg(feature = "linux_error")]
use pconst::io::FileStat;
bitflags! {
    pub struct VfsInodeMode: u32 {
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

/// Node (file/directory) type.
#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum VfsNodeType {
    Unknown = 0,
    /// FIFO (named pipe)
    Fifo = 0o1,
    /// Character device
    CharDevice = 0o2,
    /// Directory
    Dir = 0o4,
    /// Block device
    BlockDevice = 0o6,
    /// Regular file
    File = 0o10,
    /// Symbolic link
    SymLink = 0o12,
    /// Socket
    Socket = 0o14,
}

impl From<u8> for VfsNodeType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Unknown,
            0o1 => Self::Fifo,
            0o2 => Self::CharDevice,
            0o4 => Self::Dir,
            0o6 => Self::BlockDevice,
            0o10 => Self::File,
            0o12 => Self::SymLink,
            0o14 => Self::Socket,
            _ => Self::Unknown,
        }
    }
}

impl From<char> for VfsNodeType {
    fn from(value: char) -> Self {
        match value {
            '-' => Self::File,
            'd' => Self::Dir,
            'l' => Self::SymLink,
            'c' => Self::CharDevice,
            'b' => Self::BlockDevice,
            'p' => Self::Fifo,
            's' => Self::Socket,
            _ => Self::Unknown,
        }
    }
}

impl VfsNodeType {
    /// Tests whether this node type represents a regular file.
    pub const fn is_file(self) -> bool {
        matches!(self, Self::File)
    }

    /// Tests whether this node type represents a directory.
    pub const fn is_dir(self) -> bool {
        matches!(self, Self::Dir)
    }

    /// Tests whether this node type represents a symbolic link.
    pub const fn is_symlink(self) -> bool {
        matches!(self, Self::SymLink)
    }

    /// Returns `true` if this node type is a block device.
    pub const fn is_block_device(self) -> bool {
        matches!(self, Self::BlockDevice)
    }

    /// Returns `true` if this node type is a char device.
    pub const fn is_char_device(self) -> bool {
        matches!(self, Self::CharDevice)
    }

    /// Returns `true` if this node type is a fifo.
    pub const fn is_fifo(self) -> bool {
        matches!(self, Self::Fifo)
    }

    /// Returns `true` if this node type is a socket.
    pub const fn is_socket(self) -> bool {
        matches!(self, Self::Socket)
    }

    /// Returns a character representation of the node type.
    ///
    /// For example, `d` for directory, `-` for regular file, etc.
    pub const fn as_char(self) -> char {
        match self {
            Self::Fifo => 'p',
            Self::CharDevice => 'c',
            Self::Dir => 'd',
            Self::BlockDevice => 'b',
            Self::File => '-',
            Self::SymLink => 'l',
            Self::Socket => 's',
            _ => '?',
        }
    }
}

bitflags::bitflags! {
    /// Node (file/directory) permission mode.
    pub struct VfsNodePerm: u16 {
        /// Owner has read permission.
        const OWNER_READ = 0o400;
        /// Owner has write permission.
        const OWNER_WRITE = 0o200;
        /// Owner has execute permission.
        const OWNER_EXEC = 0o100;

        /// Group has read permission.
        const GROUP_READ = 0o40;
        /// Group has write permission.
        const GROUP_WRITE = 0o20;
        /// Group has execute permission.
        const GROUP_EXEC = 0o10;

        /// Others have read permission.
        const OTHER_READ = 0o4;
        /// Others have write permission.
        const OTHER_WRITE = 0o2;
        /// Others have execute permission.
        const OTHER_EXEC = 0o1;
    }
}

impl From<&str> for VfsNodePerm {
    fn from(val: &str) -> Self {
        let bytes = val.as_bytes();
        assert_eq!(bytes.len(), 9);
        let mut perm = VfsNodePerm::empty();
        if bytes[0] == b'r' {
            perm |= VfsNodePerm::OWNER_READ;
        }
        if bytes[1] == b'w' {
            perm |= VfsNodePerm::OWNER_WRITE;
        }
        if bytes[2] == b'x' {
            perm |= VfsNodePerm::OWNER_EXEC;
        }
        if bytes[3] == b'r' {
            perm |= VfsNodePerm::GROUP_READ;
        }
        if bytes[4] == b'w' {
            perm |= VfsNodePerm::GROUP_WRITE;
        }
        if bytes[5] == b'x' {
            perm |= VfsNodePerm::GROUP_EXEC;
        }
        if bytes[6] == b'r' {
            perm |= VfsNodePerm::OTHER_READ;
        }
        if bytes[7] == b'w' {
            perm |= VfsNodePerm::OTHER_WRITE;
        }
        if bytes[8] == b'x' {
            perm |= VfsNodePerm::OTHER_EXEC;
        }
        perm
    }
}
impl VfsNodePerm {
    /// Returns a 9-bytes string representation of the permission.
    ///
    /// For example, `0o755` is represented as `rwxr-xr-x`.
    pub const fn rwx_buf(&self) -> [u8; 9] {
        let mut perm = [b'-'; 9];
        if self.contains(Self::OWNER_READ) {
            perm[0] = b'r';
        }
        if self.contains(Self::OWNER_WRITE) {
            perm[1] = b'w';
        }
        if self.contains(Self::OWNER_EXEC) {
            perm[2] = b'x';
        }
        if self.contains(Self::GROUP_READ) {
            perm[3] = b'r';
        }
        if self.contains(Self::GROUP_WRITE) {
            perm[4] = b'w';
        }
        if self.contains(Self::GROUP_EXEC) {
            perm[5] = b'x';
        }
        if self.contains(Self::OTHER_READ) {
            perm[6] = b'r';
        }
        if self.contains(Self::OTHER_WRITE) {
            perm[7] = b'w';
        }
        if self.contains(Self::OTHER_EXEC) {
            perm[8] = b'x';
        }
        perm
    }
    /// Returns the default permission for a file.
    ///
    /// The default permission is `0o666` (owner/group/others can read and write).
    pub const fn default_file() -> Self {
        Self::from_bits_truncate(0o666)
    }

    /// Returns the default permission for a directory.
    ///
    /// The default permission is `0o755` (owner can read, write and execute,
    /// group/others can read and execute).
    pub const fn default_dir() -> Self {
        Self::from_bits_truncate(0o755)
    }
}

#[test]
fn test_perm_from_str() {
    let perm: VfsNodePerm = "rwxrwxrwx".into();
    assert_eq!(perm, VfsNodePerm::from_bits_truncate(0o777));
    let perm: VfsNodePerm = "rwxr-xr-x".into();
    assert_eq!(perm, VfsNodePerm::from_bits_truncate(0o755));
    let perm: VfsNodePerm = "rw-rw-rw-".into();
    assert_eq!(perm, VfsNodePerm::from_bits_truncate(0o666));
    let perm: VfsNodePerm = "rw-r--r--".into();
    assert_eq!(perm, VfsNodePerm::from_bits_truncate(0o644));
    let perm: VfsNodePerm = "rw-------".into();
    assert_eq!(perm, VfsNodePerm::from_bits_truncate(0o600));
    let perm: VfsNodePerm = "r--r--r--".into();
    assert_eq!(perm, VfsNodePerm::from_bits_truncate(0o444));
    let perm: VfsNodePerm = "r--------".into();
    assert_eq!(perm, VfsNodePerm::from_bits_truncate(0o400));
}

impl From<VfsInodeMode> for VfsNodeType {
    fn from(value: VfsInodeMode) -> Self {
        match value & VfsInodeMode::TYPE_MASK {
            VfsInodeMode::FIFO => VfsNodeType::Fifo,
            VfsInodeMode::CHAR => VfsNodeType::CharDevice,
            VfsInodeMode::DIR => VfsNodeType::Dir,
            VfsInodeMode::BLOCK => VfsNodeType::BlockDevice,
            VfsInodeMode::FILE => VfsNodeType::File,
            VfsInodeMode::LINK => VfsNodeType::SymLink,
            VfsInodeMode::SOCKET => VfsNodeType::Socket,
            _ => panic!("Invalid inode type"),
        }
    }
}

impl From<VfsInodeMode> for VfsNodePerm {
    fn from(value: VfsInodeMode) -> Self {
        VfsNodePerm::from_bits_truncate(value.bits() as u16)
    }
}

impl VfsInodeMode {
    pub fn from(perm: VfsNodePerm, ty: VfsNodeType) -> Self {
        let mut mode = VfsInodeMode::from_bits_truncate(perm.bits as u32);
        match ty {
            VfsNodeType::Fifo => mode |= VfsInodeMode::FIFO,
            VfsNodeType::CharDevice => mode |= VfsInodeMode::CHAR,
            VfsNodeType::Dir => mode |= VfsInodeMode::DIR,
            VfsNodeType::BlockDevice => mode |= VfsInodeMode::BLOCK,
            VfsNodeType::File => mode |= VfsInodeMode::FILE,
            VfsNodeType::SymLink => mode |= VfsInodeMode::LINK,
            VfsNodeType::Socket => mode |= VfsInodeMode::SOCKET,
            _ => panic!("Invalid inode type"),
        }
        mode
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn inode_mode2node_type() {
        use super::*;
        assert_eq!(VfsNodeType::Fifo, VfsInodeMode::FIFO.into());
        assert_eq!(VfsNodeType::CharDevice, VfsInodeMode::CHAR.into());
        assert_eq!(VfsNodeType::Dir, VfsInodeMode::DIR.into());
        assert_eq!(VfsNodeType::BlockDevice, VfsInodeMode::BLOCK.into());
        assert_eq!(VfsNodeType::File, VfsInodeMode::FILE.into());
        assert_eq!(VfsNodeType::SymLink, VfsInodeMode::LINK.into());
        assert_eq!(VfsNodeType::Socket, VfsInodeMode::SOCKET.into());
    }
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
pub struct VfsTimeSpec {
    pub sec: u64,  /* 秒 */
    pub nsec: u64, /* 纳秒, 范围在0~999999999 */
}

impl VfsTimeSpec {
    pub fn new(sec: u64, nsec: u64) -> Self {
        Self { sec, nsec }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum VfsTime {
    AccessTime(VfsTimeSpec),
    ModifiedTime(VfsTimeSpec),
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VfsFsStat {
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

impl Default for VfsFsStat {
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

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
#[repr(C)]
pub struct VfsFileStat {
    pub st_dev: u64,
    pub st_ino: u64,
    pub st_mode: u32,
    pub st_nlink: u32,
    pub st_uid: u32,
    pub st_gid: u32,
    pub st_rdev: u64,
    pub __pad: u64,
    pub st_size: u64,
    pub st_blksize: u32,
    pub __pad2: u32,
    pub st_blocks: u64,
    pub st_atime: VfsTimeSpec,
    pub st_mtime: VfsTimeSpec,
    pub st_ctime: VfsTimeSpec,
    pub unused: u64,
} //128

#[cfg(feature = "linux_error")]
impl From<VfsFileStat> for FileStat {
    fn from(value: VfsFileStat) -> Self {
        Self {
            st_dev: value.st_dev,
            st_ino: value.st_ino,
            st_mode: value.st_mode,
            st_nlink: value.st_nlink,
            st_uid: value.st_uid,
            st_gid: value.st_gid,
            st_rdev: value.st_rdev,
            __pad: 0,
            st_size: value.st_size,
            st_blksize: value.st_blksize,
            __pad2: 0,
            st_blocks: value.st_blocks,
            st_atime_sec: value.st_atime.sec,
            st_atime_nsec: value.st_atime.nsec,
            st_mtime_sec: value.st_mtime.sec,
            st_mtime_nsec: value.st_mtime.nsec,
            st_ctime_sec: value.st_ctime.nsec,
            st_ctime_nsec: value.st_ctime.nsec,
            unused: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VfsDirEntry {
    /// ino is an inode number
    pub ino: u64,
    /// type is the file type
    pub ty: VfsNodeType,
    /// filename (null-terminated)
    pub name: String,
}

bitflags! {
    /// ppoll 使用，表示对应在文件上等待或者发生过的事件
    pub struct VfsPollEvents: u16 {
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

bitflags! {
    pub struct VfsMountFlags:u32{
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
     /// renameat flag
    pub struct VfsRenameFlag: u32 {
        /// Atomically exchange oldpath and newpath.
        /// Both pathnames must exist but may be of different type
        const RENAME_EXCHANGE = 1 << 1;
        /// Don't overwrite newpath of the rename. Return an error if newpath already exists.
        const RENAME_NOREPLACE = 1 << 0;
        /// This operation makes sense only for overlay/union filesystem implementations.
        const RENAME_WHITEOUT = 1 << 2;
    }
}
