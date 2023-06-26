use Mstd::fs::{close, list, mkdir, mkdirat, open, OpenFlags, read, renameat, seek, write};
use Mstd::println;

pub fn dir_test() -> isize {
    fat32_test();
    dbfs_test();
    0
}

fn fat32_test() {
    println!("In this test, we will test mkdirat and renameat");
    let res = mkdirat(0, "/dir1\0", OpenFlags::O_RDWR);
    if res == -1 {
        println!("mkdirat error");
    }
    println!("mkdir /dir1 success");
    let fd = open("/dir1/f1\0", OpenFlags::O_CREAT | OpenFlags::O_RDWR);
    if fd == -1 {
        println!("open error");
    }

    let len = write(fd as usize, "hello world".as_bytes());
    if len == -1 {
        println!("write error");
    }
    println!("write {} bytes to f1", len);

    println!("create /dir1/f1 success");
    list("/dir1");
    let res = renameat(0, "/dir1/f1\0", 0, "/dir1/f2\0");
    if res == -1 {
        println!("renameat error");
    }
    println!("rename /dir1/f1 to /dir1/f2 success");
    list("/dir1");

    seek(fd as usize, 0, 0);
    let mut buf = [0u8; 20];
    let len = read(fd as usize, &mut buf);
    if len == -1 {
        println!("read error");
    }
    println!("read {} bytes from f1", len);
    println!(
        "read buf:{}",
        core::str::from_utf8(&buf[..len as usize]).unwrap()
    );
    close(fd as usize);
}

fn dbfs_test() {
    println!("In this test, we will test mkdirat and renameat");
    let res = mkdirat(0, "/db/dir1\0", OpenFlags::O_RDWR);
    if res == -1 {
        println!("mkdirat error");
    }
    println!("mkdir /db/dir1 success");
    let fd = open("/db/dir1/f1\0", OpenFlags::O_CREAT | OpenFlags::O_RDWR);
    if fd == -1 {
        println!("open error");
    }

    let len = write(fd as usize, "hello world".as_bytes());
    if len == -1 {
        println!("write error");
    }
    println!("write {} bytes to f1", len);

    println!("create /db/dir1/f1 success");
    list("/db/dir1");
    let res = renameat(0, "/db/dir1/f1\0", 0, "/db/dir1/f2\0");
    if res == -1 {
        println!("renameat error");
    }
    println!("rename /db/dir1/f1 to /db/dir1/f2 success");
    list("/db/dir1");

    seek(fd as usize, 0, 0);
    let mut buf = [0u8; 20];
    let len = read(fd as usize, &mut buf);
    if len == -1 {
        println!("read error");
    }
    println!("read {} bytes from f1", len);
    println!(
        "read buf:{}",
        core::str::from_utf8(&buf[..len as usize]).unwrap()
    );
    close(fd as usize);
}
