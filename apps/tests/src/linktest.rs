use Mstd::fs::{close, fstat, linkat, LinkFlags, open, OpenFlags, readlinkat, Stat, symlinkat, unlinkat};

pub fn link_test() -> isize {
    let fd = open("/db/linktest\0", OpenFlags::O_CREAT | OpenFlags::O_WRONLY);
    println!("fd = {}", fd);
    if fd == -1 {
        println!("open error");
    }
    let res = linkat(
        fd,
        "/db/linktest\0",
        fd as usize,
        "/db/linktest1\0",
        LinkFlags::empty(),
    );
    println!("linkat = {}", res);
    let mut stat = Stat::default();
    let fd = fd as usize;
    let _r = fstat(fd, &mut stat);
    let res = unlinkat(fd as isize, "/db/linktest1\0", 0);
    println!("unlinkat = {}", res);
    let _r = fstat(fd, &mut stat);
    println!("link:{:#?}", stat);

    let res = symlinkat("/db/linktest\0", fd as isize, "/db/linktest2\0");
    // let _r = fstat(fd,&mut stat);
    let symlink_fd = open("/db/linktest2\0", OpenFlags::O_RDONLY);
    println!("symlink_fd = {}", symlink_fd);
    let mut buf = [0u8; 100];
    let len = readlinkat(0, "/db/linktest2\0", &mut buf);
    println!("readlinkat len :{}", len);
    println!(
        "readlinkat buf :{}",
        core::str::from_utf8(&buf[..len as usize]).unwrap()
    );
    close(fd);
    close(symlink_fd as usize);
    0
}