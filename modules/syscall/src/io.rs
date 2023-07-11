use bitflags::bitflags;

#[derive(Debug, Clone)]
pub struct IoVec {
    pub base: *mut u8,
    pub len: usize,
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
        F_DUPFD_CLOEXEC = 1030,
        Unknown = 0xffff,
    }
}

bitflags! {
    pub struct MapFlags: u32 {
        const MAP_SHARED = 0x01;
        const MAP_PRIVATE = 0x02;
        const MAP_FIXED = 0x10;
        const MAP_ANONYMOUS = 0x20;
        // 映射时不保留空间，即可能在实际使用mmp出来的内存时内存溢出
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
