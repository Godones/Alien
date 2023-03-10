mod dbfs;
mod stdio;

use crate::fs::vfs::VfsProvider;
use crate::task::current_process;
use rvfs::file::{vfs_open_file, vfs_read_file, vfs_readdir, vfs_write_file, FileFlags, FileMode};
pub use stdio::*;
pub mod vfs;
// type UserBuffer = Vec<&'static mut [u8]>;

pub fn sys_read(fd: usize, buf: *mut u8, len: usize) -> isize {
    let process = current_process().unwrap();
    let file = process.get_file(fd);
    if file.is_none() {
        return -1;
    }
    let file = file.unwrap();
    let mut buf = process.transfer_raw_buffer(buf, len);
    let mut count = 0;
    let offset = file.access_inner().f_pos;
    buf.iter_mut().for_each(|b| {
        let r = vfs_read_file::<VfsProvider>(file.clone(), b, offset as u64).unwrap();
        count += r;
    });
    count as isize
}

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let process = current_process().unwrap();
    let file = process.get_file(fd);
    if file.is_none() {
        return -1;
    }
    let file = file.unwrap();
    let mut buf = process.transfer_raw_buffer(buf, len);
    let mut count = 0;
    let offset = file.access_inner().f_pos;
    buf.iter_mut().for_each(|b| {
        let r = vfs_write_file::<VfsProvider>(file.clone(), b, offset as u64).unwrap();
        count += r;
    });
    count as isize
}

pub fn sys_list() -> isize {
    list_dir();
    0
}

// temp function, will be removed
pub fn list_dir() {
    let file = vfs_open_file::<VfsProvider>("/", FileFlags::O_RDWR, FileMode::FMODE_READ).unwrap();
    vfs_readdir(file).unwrap().for_each(|x| {
        println!("name: {}", x);
    })
}
