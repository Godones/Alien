//! use fat32

mod dbfs;
mod fat32;
mod stdio;

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::error::Error;
use core::fmt::Debug;
use fat32_trait::{DirectoryLike, FileLike};
pub use stdio::*;

#[cfg(feature = "dbfs")]
use crate::fs::dbfs::ROOT_DIR;
#[cfg(feature = "fat32")]
use crate::fs::fat32::ROOT_DIR;
use crate::task::current_process;

type UserBuffer = Vec<&'static mut [u8]>;
pub trait File: Send + Sync + Debug {
    fn write(&self, buf: UserBuffer) -> usize;
    fn read(&self, buf: UserBuffer) -> usize;
}

pub fn sys_read(fd: usize, buf: *mut u8, len: usize) -> isize {
    let process = current_process().unwrap();
    let file = process.get_file(fd);
    if file.is_none() {
        return -1;
    }
    let file = file.unwrap();
    let buf = process.transfer_raw_buffer(buf, len);
    file.read(buf) as isize
}

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let process = current_process().unwrap();
    let file = process.get_file(fd);
    if file.is_none() {
        return -1;
    }
    let file = file.unwrap();
    let buf = process.transfer_raw_buffer(buf, len);
    file.write(buf) as isize
}

// temp function, will be removed
pub fn list_dir() {
    let root = ROOT_DIR.clone();
    println!("---------APP LIST---------");
    root.list().unwrap().iter().for_each(|entry| {
        println!("{}", entry);
    });
}

pub fn open_file(path: &str) -> Option<Arc<dyn FileLike<Error: Error + 'static>>> {
    let root = ROOT_DIR.clone();
    let file = root.open(path).unwrap();
    Some(file)
}
