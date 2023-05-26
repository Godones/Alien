use core::arch::asm;

use crate::{syscall, syscall_id};

syscall_id!(SYSCALL_SETXATTR, 5);
syscall_id!(SYSCALL_LSETXATTR, 6);
syscall_id!(SYSCALL_FSETXATTR, 7);
syscall_id!(SYSCALL_GETXATTR, 8);
syscall_id!(SYSCALL_LGETXATTR, 9);
syscall_id!(SYSCALL_FGETXATTR, 10);
syscall_id!(SYSCALL_LISTXATTR, 11);
syscall_id!(SYSCALL_LLISTXATTR, 12);
syscall_id!(SYSCALL_FLISTXATTR, 13);
syscall_id!(SYSCALL_REMOVEXATTR, 14);
syscall_id!(SYSCALL_LREMOVEXATTR, 15);
syscall_id!(SYSCALL_FREMOVEXATTR, 16);

syscall_id!(SYSCALL_GETCWD, 17);

syscall_id!(SYSCALL_DUP, 23);
syscall_id!(SYSCALL_DUP3, 24);
syscall_id!(SYSCALL_LINKAT, 37);
syscall_id!(SYSCALL_UNLINKAT, 35);
syscall_id!(SYSCALL_SYMLINKAT, 36);
syscall_id!(SYSCALL_READLINKAT, 78);
syscall_id!(SYSCALL_CHDIR, 49);
syscall_id!(SYSCALL_READ, 63);
syscall_id!(SYSCALL_FSTATFS, 44);
syscall_id!(SYSCALL_STATFS, 43);
syscall_id!(SYSCALL_TRUNCATE, 45);
syscall_id!(SYSCALL_FTRUNCATE, 46);
syscall_id!(SYSCALL_PIPE, 59);

syscall_id!(SYSCALL_GETDENTS, 61);
syscall_id!(SYSCALL_WRITE, 64);
syscall_id!(SYSCALL_FSTAT, 80);
syscall_id!(SYSCALL_FSTATAT, 79);
syscall_id!(SYSCALL_EXIT, 93);
syscall_id!(SYSCALL_YIELD, 124);
syscall_id!(SYSCALL_GET_TIME, 169);
syscall_id!(SYSCALL_GETPID, 172);
syscall_id!(SYSCALL_FORK, 220);
syscall_id!(SYSCALL_EXEC, 221);
syscall_id!(SYSCALL_WAITPID, 260);
syscall_id!(SYSCALL_SHUTDOWN, 210);

syscall_id!(SYSCALL_OPENAT, 56);

syscall_id!(SYSCALL_CLOSE, 57);
syscall_id!(SYSCALL_LSEEK, 62);
syscall_id!(SYSCALL_MKDIR, 83);
syscall_id!(SYSCALL_RMDIR, 84);
syscall_id!(SYSCALL_UNLINK, 87);
syscall_id!(SYSCALL_RENAMEAT, 38);
syscall_id!(SYSCALL_MKDIRAT, 34);

syscall_id!(SYSCALL_BRK, 214);
syscall_id!(SYSCALL_NANO_SLEEP, 101);

syscall_id!(SYSCALL_LIST, 1000);
syscall_id!(SYSCALL_CREATE_GLOBAL_BUCKET, 1001);
syscall_id!(SYSCALL_EXECUTE_USER_FUNC, 1002);
syscall_id!(SYSCALL_SHOW_DBFS, 1003);
syscall_id!(SYSCALL_EXECUTE_OPERATE, 1004);
fn syscall(id: usize, args: [usize; 6]) -> isize {
    let mut ret: isize;
    unsafe {
        asm!(
        "ecall",
        inlateout("x10") args[0] => ret,
        in("x11") args[1],
        in("x12") args[2],
        in("x13") args[3],
        in("x14") args[4],
        in("x15") args[5],
        in("x17") id
        );
    }
    ret
}

syscall!(sys_read, SYSCALL_READ, usize, *mut u8, usize);
syscall!(sys_write, SYSCALL_WRITE, usize, *const u8, usize);
syscall!(sys_exit, SYSCALL_EXIT, i32);
syscall!(sys_yield, SYSCALL_YIELD);
syscall!(sys_getpid, SYSCALL_GETPID);
syscall!(sys_get_time, SYSCALL_GET_TIME, *mut u8);
syscall!(sys_fork, SYSCALL_FORK);
syscall!(sys_execve, SYSCALL_EXEC, *const u8, *const usize);
syscall!(sys_waitpid, SYSCALL_WAITPID, isize, *mut i32);
syscall!(sys_shutdown, SYSCALL_SHUTDOWN);
syscall!(sys_list, SYSCALL_LIST, *const u8);
syscall!(sys_openat, SYSCALL_OPENAT, isize, *const u8, usize, usize);
syscall!(sys_close, SYSCALL_CLOSE, usize);
syscall!(sys_get_cwd, SYSCALL_GETCWD, *mut u8, usize);
syscall!(sys_chdir, SYSCALL_CHDIR, *const u8);
syscall!(sys_mkdir, SYSCALL_MKDIR, *const u8);
syscall!(sys_nanosleep, SYSCALL_NANO_SLEEP, *mut u8, *mut u8);

syscall!(
    sys_create_global_bucket,
    SYSCALL_CREATE_GLOBAL_BUCKET,
    *const u8
);
syscall!(
    sys_execute_user_func,
    SYSCALL_EXECUTE_USER_FUNC,
    *const u8,
    *const u8,
    usize,
    usize
);
syscall!(sys_show_dbfs, SYSCALL_SHOW_DBFS);
syscall!(
    sys_dbfs_execute_operate,
    SYSCALL_EXECUTE_OPERATE,
    *const u8,
    *const u8
);

syscall!(sys_lseek, SYSCALL_LSEEK, usize, isize, usize);
syscall!(sys_fstat, SYSCALL_FSTAT, usize, *mut u8);
syscall!(
    sys_linkat,
    SYSCALL_LINKAT,
    isize,
    *const u8,
    usize,
    *const u8,
    usize
);
syscall!(sys_unlinkat, SYSCALL_UNLINKAT, isize, *const u8, usize);
syscall!(
    sys_symlinkat,
    SYSCALL_SYMLINKAT,
    *const u8,
    isize,
    *const u8
);
syscall!(
    sys_readlinkat,
    SYSCALL_READLINKAT,
    isize,
    *const u8,
    *mut u8,
    usize
);
syscall!(
    sys_fstatat,
    SYSCALL_FSTATAT,
    isize,
    *const u8,
    *mut u8,
    usize
);
syscall!(sys_fstatfs, SYSCALL_FSTATFS, usize, *mut u8);
syscall!(sys_statfs, SYSCALL_STATFS, *const u8, *mut u8);
syscall!(sys_mkdirat, SYSCALL_MKDIRAT, isize, *const u8, usize);
syscall!(
    sys_renameat,
    SYSCALL_RENAMEAT,
    isize,
    *const u8,
    isize,
    *const u8
);

syscall!(
    sys_setxattr,
    SYSCALL_SETXATTR,
    *const u8,
    *const u8,
    *const u8,
    usize,
    usize
);
syscall!(
    sys_lsetxattr,
    SYSCALL_LSETXATTR,
    *const u8,
    *const u8,
    *const u8,
    usize,
    usize
);
syscall!(
    sys_fsetxattr,
    SYSCALL_FSETXATTR,
    usize,
    *const u8,
    *const u8,
    usize,
    usize
);

syscall!(
    sys_getxattr,
    SYSCALL_GETXATTR,
    *const u8,
    *const u8,
    *mut u8,
    usize
);
syscall!(
    sys_lgetxattr,
    SYSCALL_LGETXATTR,
    *const u8,
    *const u8,
    *mut u8,
    usize
);
syscall!(
    sys_fgetxattr,
    SYSCALL_FGETXATTR,
    usize,
    *const u8,
    *mut u8,
    usize
);

syscall!(sys_listxattr, SYSCALL_LISTXATTR, *const u8, *mut u8, usize);
syscall!(
    sys_llistxattr,
    SYSCALL_LLISTXATTR,
    *const u8,
    *mut u8,
    usize
);
syscall!(sys_flistxattr, SYSCALL_FLISTXATTR, usize, *mut u8, usize);

syscall!(sys_removexattr, SYSCALL_REMOVEXATTR, *const u8, *const u8);
syscall!(sys_lremovexattr, SYSCALL_LREMOVEXATTR, *const u8, *const u8);
syscall!(sys_fremovexattr, SYSCALL_FREMOVEXATTR, usize, *const u8);
syscall!(sys_getdents, SYSCALL_GETDENTS, *const u8, *mut u8, usize);

syscall!(sys_truncate, SYSCALL_TRUNCATE, *const u8, usize);
syscall!(sys_ftruncate, SYSCALL_FTRUNCATE, usize, usize);

// ipc
syscall!(sys_pipe, SYSCALL_PIPE, *mut u32, usize);
syscall!(sys_dup, SYSCALL_DUP, usize);
syscall!(sys_dup3, SYSCALL_DUP3, usize, usize, usize);

// alloc
syscall!(sys_brk, SYSCALL_BRK, usize);
