#![no_std]
#![no_main]

extern crate alloc;

use alloc::{string::String, vec::Vec};

use Mstd::{
    fs::{getdents, open, Dirent64, DirentType, OpenFlags},
    println,
};

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
const BUF_SIZE: usize = 512;
fn parse_args(path: &str) {
    let fd = open(path, OpenFlags::O_RDONLY);
    assert!(fd >= 0, "open failed");
    let mut buf = [0u8; BUF_SIZE];
    loop {
        let size = getdents(fd as usize, &mut buf);
        // assert!(size >= 0, "getdents failed");
        if size == 0 || size == -1 {
            break;
        }
        let mut ptr = buf.as_ptr();
        let mut count = 0;
        loop {
            let dirent = unsafe { &*(ptr as *const Dirent64) };
            println!("{} {}", trans(dirent.type_), dirent.get_name());
            count += dirent.len();
            if count >= size as usize {
                break;
            }
            ptr = unsafe { ptr.add(dirent.len()) };
        }
        buf.fill(0);
    }
}

fn trans(type_: DirentType) -> char {
    match type_ {
        DirentType::DT_UNKNOWN => '?',
        DirentType::DT_FIFO => 'p',
        DirentType::DT_CHR => 'c',
        DirentType::DT_DIR => 'd',
        DirentType::DT_BLK => 'b',
        DirentType::DT_REG => '-',
        DirentType::DT_LNK => 'l',
        DirentType::DT_SOCK => 's',
        DirentType::DT_WHT => 'w',
        _ => '?',
    }
}
