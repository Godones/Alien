#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use Mstd::fs::{getdents, open, Dirent64, OpenFlags};
use Mstd::println;

#[no_mangle]
fn main(_: usize, argv: Vec<String>) -> isize {
    if argv.len() == 1 {
        parse_args("./\0")
    } else {
        let mut path = argv[1].clone();
        path.push('\0');
        parse_args(path.as_str())
    }
    0
}

fn parse_args(path: &str) {
    let fd = open(path, OpenFlags::O_RDONLY);
    assert!(fd > 0, "open failed");
    let size = getdents(fd as usize, &mut []);
    assert!(size > 0, "getdents failed");
    let mut buf = vec![0u8; size as usize];
    let size = getdents(fd as usize, buf.as_mut_slice());
    let mut ptr = buf.as_ptr() as *const u8;
    let mut count = 0;
    loop {
        let dirent = unsafe { &*(ptr as *const Dirent64) };
        println!("{} {}", dirent.type_.to_string(), dirent.get_name());
        count += dirent.len();
        if count >= size as usize {
            break;
        }
        ptr = unsafe { ptr.add(dirent.len()) };
    }
}
