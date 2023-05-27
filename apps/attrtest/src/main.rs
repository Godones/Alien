#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use Mstd::fs::{close, fgetxattr, flistxattr, fsetxattr, open, OpenFlags};
use Mstd::println;

#[no_mangle]
fn main() -> isize {
    println!("In this test, we will test attr");
    let fd = open("/db/attrtest\0", OpenFlags::O_RDWR | OpenFlags::O_CREAT);
    if fd == -1 {
        println!("open error");
        return -1;
    }
    println!("open file /db/attrtest fd = {}", fd);
    // setxattr()
    let r = fsetxattr(fd as usize, "user.test\0", "file attr1".as_bytes(), 0);
    if r == -1 {
        println!("fsetxattr error");
        return -1;
    }
    let r1 = fsetxattr(fd as usize, "user.foo\0", "file attr2".as_bytes(), 0);
    if r1 == -1 {
        println!("fsetxattr error");
        return -1;
    }
    let mut zero_buf = [0u8; 0];
    // println!("buf size:{}",zero_buf.len());
    let len = flistxattr(fd as usize, &mut zero_buf);
    println!(
        "The length of the list of extended attribute names associated with the file is {}",
        len
    );
    let mut buf = vec![0u8; len as usize];
    let r2 = flistxattr(fd as usize, &mut buf);
    if r2 == -1 {
        println!("flistxattr error");
        return -1;
    }
    // println!("flistxattr:{:#?}",buf);
    buf.as_slice()[..r2 as usize - 1]
        .split(|&x| x == 0)
        .collect::<Vec<&[u8]>>()
        .into_iter()
        .for_each(|x| {
            println!("attr name: {}", core::str::from_utf8(x).unwrap());
            let mut zero_buf = [0u8; 0];
            let mut name = x.to_vec();
            name.push(0);
            let len = fgetxattr(
                fd as usize,
                core::str::from_utf8(name.as_slice()).unwrap(),
                &mut zero_buf,
            );
            assert_ne!(len, -1);
            if len != 0 {
                println!("attr value len:{}", len);
                let mut buf = vec![0u8; len as usize];
                let r3 = fgetxattr(
                    fd as usize,
                    core::str::from_utf8(name.as_slice()).unwrap(),
                    &mut buf,
                );
                if r3 == -1 {
                    println!("fgetxattr error");
                    return;
                }
                println!("attr value:{}", core::str::from_utf8(&buf).unwrap());
            }
        });
    close(fd as usize);
    0
}
