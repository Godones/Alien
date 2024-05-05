use alloc::{vec, vec::Vec};

use Mstd::fs::{
    close, fgetxattr, flistxattr, fsetxattr, fstat, ftruncate, open, read, seek, write, OpenFlags,
    Stat,
};

pub fn attr_test1() -> isize {
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

pub fn attr_test2() -> isize {
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
    close(fd as usize);
    0
}
