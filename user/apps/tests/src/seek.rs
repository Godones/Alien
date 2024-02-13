use Mstd::fs::{close, fstat, open, read, seek, write, OpenFlags, Stat};
use Mstd::println;

pub fn seek_test() -> isize {
    let fd = open("/test.txt\0", OpenFlags::O_CREAT | OpenFlags::O_WRONLY);
    println!("fd = {}", fd);
    if fd != -1 {
        let w = write(fd as usize, "hello world".as_bytes());
        println!("write = {}", w);
        let v = seek(fd as usize, 6, 0);
        if v != -1 {
            let mut buf = [0u8; 10];
            let r = read(fd as usize, &mut buf);
            println!("read = {}", r);
            println!(
                "buf = {:?}",
                core::str::from_utf8(&buf[..r as usize]).unwrap()
            );
        }
    }
    let mut stat = Stat::default();
    let fd = fd as usize;
    let r = fstat(fd, &mut stat);
    println!("stat:{:#?}", stat);
    close(fd);
    0
}
