## filesystem syscall support
Now the os supports the following system calls:

```rust
pub fn sys_openat(dirfd: isize, path: usize, flag: usize, mode: usize) -> isize
pub fn sys_close(fd: usize) -> isize
pub fn sys_read(fd: usize, buf: *mut u8, len: usize) -> isize
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize
pub fn sys_getcwd(buf: *mut u8, len: usize) -> isize 
pub fn sys_chdir(path: *const u8) -> isize
pub fn sys_mkdir(path: *const u8) -> isize
pub fn sys_list(path: *const u8) -> isize
pub fn sys_lseek(fd: usize, offset: isize, whence: usize) -> isize 
pub fn sys_fstat(fd: usize, stat: *mut u8) -> isize
pub fn sys_linkat(
    old_fd: isize,
    old_name: *const u8,
    new_fd: isize,
    new_name: *const u8,
    flag: usize,
) -> isize
pub fn sys_unlinkat(fd: isize, path: *const u8, flag: usize) -> isize
pub fn sys_symlinkat(old_name: *const u8, new_fd: isize, new_name: *const u8) -> isize
pub fn sys_readlinkat(fd: isize, path: *const u8, buf: *mut u8, size: usize) -> isize
pub fn sys_fstateat(dir_fd: isize, path: *const u8, stat: *mut u8, flag: usize) -> isize 
pub fn sys_fstatfs(fd: isize, buf: *mut u8) -> isize
pub fn sys_statfs(path: *const u8, statfs: *const u8) -> isize 
pub fn sys_renameat(
    old_dirfd: isize,
    old_path: *const u8,
    new_dirfd: isize,
    new_path: *const u8,
) -> isize
pub fn sys_mkdirat(dirfd: isize, path: *const u8, flag: usize) -> isize
pub fn sys_setxattr(
    path: *const u8,
    name: *const u8,
    value: *const u8,
    size: usize,
    flag: usize,
) -> isize
pub fn sys_getxattr(path: *const u8, name: *const u8, value: *const u8, size: usize) -> isize
pub fn sys_fgetxattr(fd: usize, name: *const u8, value: *const u8, size: usize) -> isize
pub fn sys_listxattr(path: *const u8, list: *const u8, size: usize) -> isize
pub fn sys_flistxattr(fd: usize, list: *const u8, size: usize) -> isize
pub fn sys_removexattr(path: *const u8, name: *const u8) -> isize
pub fn sys_fremovexattr(fd: usize, name: *const u8) -> isize
```

There are some applications in the `app` directory to test the system calls.
