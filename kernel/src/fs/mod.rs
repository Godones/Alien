mod dbfs;
mod stdio;
use crate::fs::vfs::VfsProvider;
use crate::task::current_process;
use core::cmp::min;
use rvfs::file::{
    vfs_llseek, vfs_mkdir, vfs_open_file, vfs_read_file, vfs_readdir, vfs_write_file, FileMode,
    OpenFlags, SeekFrom,
};

use rvfs::inode::InodeMode;
pub use stdio::*;
use syscall_table::syscall_func;
pub mod vfs;
pub use dbfs::{
    init_dbfs, sys_create_global_bucket, sys_execute_user_func, sys_execute_user_operate,
    sys_show_dbfs,
};

#[syscall_func(56)]
pub fn sys_open(path: usize, flag: u32) -> isize {
    let process = current_process().unwrap();
    let path = process.transfer_str(path as *const u8);
    let file = vfs_open_file::<VfsProvider>(
        &path,
        OpenFlags::from_bits_truncate(flag),
        FileMode::FMODE_READ | FileMode::FMODE_WRITE,
    );
    if file.is_err() {
        return -1;
    }
    let fd = process.add_file(file.unwrap());
    if fd.is_err() {
        -1
    } else {
        fd.unwrap() as isize
    }
}
#[syscall_func(57)]
pub fn sys_close(fd: usize) -> isize {
    let process = current_process().unwrap();
    let fd = process.remove_file(fd);
    if fd.is_err() {
        return -1;
    }
    0
}
#[syscall_func(63)]
pub fn sys_read(fd: usize, buf: *mut u8, len: usize) -> isize {
    let process = current_process().unwrap();
    let file = process.get_file(fd);
    if file.is_none() {
        return -1;
    }
    let file = file.unwrap();
    let mut buf = process.transfer_raw_buffer(buf, len);
    let mut count = 0;
    let mut offset = file.access_inner().f_pos;
    buf.iter_mut().for_each(|b| {
        let r = vfs_read_file::<VfsProvider>(file.clone(), b, offset as u64).unwrap();
        count += r;
        offset += r;
    });
    count as isize
}

#[syscall_func(64)]
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    // if fd==3{
    //     println!("sys_write({},{},{})", fd, buf as usize, len);
    // }

    let process = current_process().unwrap();
    let file = process.get_file(fd);
    if file.is_none() {
        return -1;
    }
    let file = file.unwrap();
    let mut buf = process.transfer_raw_buffer(buf, len);
    let mut count = 0;
    let mut offset = file.access_inner().f_pos;
    buf.iter_mut().for_each(|b| {
        let r = vfs_write_file::<VfsProvider>(file.clone(), b, offset as u64).unwrap();
        count += r;
        offset += r;
        // if fd == 3{
        //     println!("write {} bytes, buf len :{}", r,b.len());
        // }

    });
    count as isize
}
#[syscall_func(17)]
pub fn sys_getcwd(buf: *mut u8, len: usize) -> isize {
    let process = current_process().unwrap();
    let cwd = process.access_inner().cwd();
    let mut buf = process.transfer_raw_buffer(buf, len);
    let mut count = 0;
    let mut cwd = cwd.as_bytes();
    buf.iter_mut().for_each(|buf| {
        // fill buf
        if !cwd.is_empty() {
            let min = min(cwd.len(), buf.len());
            buf[..min].copy_from_slice(&cwd[..min]);
            count += min;
            cwd = &cwd[min..];
        }
    });
    count as isize
}

#[syscall_func(49)]
pub fn sys_chdir(path: *const u8) -> isize {
    let process = current_process().unwrap();
    let path = process.transfer_str(path);
    let file = vfs_open_file::<VfsProvider>(
        path.as_str(),
        OpenFlags::O_RDWR | OpenFlags::O_DIRECTORY,
        FileMode::FMODE_READ,
    );
    if file.is_err() {
        return -1;
    }
    let lookup = file.unwrap();

    if lookup.f_dentry.access_inner().d_inode.mode != InodeMode::S_DIR {
        return -1;
    }
    process.access_inner().fs_info.cwd = lookup.f_dentry.clone();
    process.access_inner().fs_info.cmnt = lookup.f_mnt.clone();
    0
}

#[syscall_func(83)]
pub fn sys_mkdir(path: *const u8) -> isize {
    info!("sys_mkdir");
    let process = current_process().unwrap();
    let path = process.transfer_str(path);
    let file = vfs_mkdir::<VfsProvider>(&path, FileMode::FMODE_WRITE);
    if file.is_err() {
        return -1;
    }
    0
}

#[syscall_func(1000)]
pub fn sys_list(path: *const u8) -> isize {
    let process = current_process().unwrap();
    let path = process.transfer_str(path);
    do_list(path.as_str())
}

fn do_list(path: &str) -> isize {
    let file = vfs_open_file::<VfsProvider>(
        path,
        OpenFlags::O_RDWR | OpenFlags::O_DIRECTORY,
        FileMode::FMODE_READ,
    );
    if file.is_err() {
        return -1;
    }
    vfs_readdir(file.unwrap()).unwrap().for_each(|x| {
        println!("name: {}", x);
    });
    0
}

#[syscall_func(62)]
pub fn sys_lseek(fd: usize, offset: isize, whence: usize) -> isize {
    let process = current_process().unwrap();
    let file = process.get_file(fd);
    if file.is_none() {
        return -1;
    }
    let file = file.unwrap();
    let seek = SeekFrom::from((whence, offset as usize));
    let res = vfs_llseek(file, seek);
    if res.is_err() {
        return -1;
    }
    0
}
