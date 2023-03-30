use crate::{syscall, syscall_id};
use core::arch::asm;
syscall_id!(SYSCALL_GETCWD, 17);
syscall_id!(SYSCALL_CHDIR, 49);
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
syscall_id!(SYSCALL_OPEN, 56);
syscall_id!(SYSCALL_CLOSE, 57);
syscall_id!(SYSCALL_LSEEK, 62);
syscall_id!(SYSCALL_MKDIR, 83);
syscall_id!(SYSCALL_RMDIR, 84);
syscall_id!(SYSCALL_UNLINK, 87);
syscall_id!(SYSCALL_LIST, 1000);

syscall_id!(SYSCALL_CREATE_GLOBAL_BUCKET, 1001);
syscall_id!(SYSCALL_EXECUTE_USER_FUNC, 1002);
syscall_id!(SYSCALL_SHOW_DBFS, 1003);
syscall_id!(SYSCALL_EXECUTE_OPERATE, 1004);
syscall_id!(SYSCALL_SLEEP, 1005);
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
syscall!(sys_get_time, SYSCALL_GET_TIME);
syscall!(sys_fork, SYSCALL_FORK);
syscall!(sys_execve, SYSCALL_EXEC, *const u8, *const usize);
syscall!(sys_waitpid, SYSCALL_WAITPID, isize, *mut i32);
syscall!(sys_shutdown, SYSCALL_SHUTDOWN);
syscall!(sys_list, SYSCALL_LIST, *const u8);
syscall!(sys_open, SYSCALL_OPEN, *const u8, usize);
syscall!(sys_close, SYSCALL_CLOSE, usize);
syscall!(sys_get_cwd, SYSCALL_GETCWD, *mut u8, usize);
syscall!(sys_chdir, SYSCALL_CHDIR, *const u8);
syscall!(sys_mkdir, SYSCALL_MKDIR, *const u8);

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

syscall!(sys_sleep, SYSCALL_SLEEP, usize);
