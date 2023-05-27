#![no_main]
#![no_std]

use Mstd::fs::{fstat, ftruncate, open, read, seek, write, OpenFlags, Stat};
use Mstd::println;

#[no_mangle]
fn main() {
    println!("We will test getdents and truncate....");
    let fd = open("/db/attrtest\0", OpenFlags::O_RDWR | OpenFlags::O_CREAT);
    assert_ne!(fd, -1);
    let len = write(fd as usize, "hello world".as_bytes());
    assert_ne!(len, -1);
    println!("write {} bytes", len);
    let mut stat = Stat::default();
    fstat(fd as usize, &mut stat);
    println!("size:{}", stat.st_size);
    let r = ftruncate(fd as usize, 5);
    assert_ne!(r, -1);
    fstat(fd as usize, &mut stat);
    println!("size:{}", stat.st_size);
    let mut buf = [0u8; 20];
    let r = read(fd as usize, &mut buf);
    assert_eq!(r, 0);
    let len = write(fd as usize, "hello world".as_bytes());
    assert_ne!(len, -1);
    println!("write {} bytes", len);
    fstat(fd as usize, &mut stat);
    println!("size:{}", stat.st_size);
    seek(fd as usize, 0, 0);
    let mut buf = [0u8; 30];
    let r = read(fd as usize, &mut buf);
    assert_ne!(r, -1);
    println!("read {} bytes", r);
    assert_eq!(r, 22);
    println!(
        "buf: {}",
        core::str::from_utf8(&buf[0..r as usize]).unwrap()
    );
    // println!("buf: {:?}", &buf[..22]);
}
