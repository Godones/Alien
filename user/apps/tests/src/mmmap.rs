use Mstd::{
    fs::{close, fstat, open, write, OpenFlags, Stat},
    ipc::{mmap, munmap, MapFlags, ProtFlags},
};

pub fn mmap_test() {
    println!("Test mmap and munmap");
    let str = "Hello, mmap successfully!";
    let fd = open("test_mmap.txt\0", OpenFlags::O_RDWR | OpenFlags::O_CREAT);
    assert!(fd > 0);
    write(fd as usize, str.as_bytes());
    let mut stat = Stat::default();
    let res = fstat(fd as usize, &mut stat);
    assert_eq!(res, 0);
    println!("The file size is {}", stat.st_size);

    let start = mmap(
        0,
        stat.st_size as usize,
        ProtFlags::PROT_WRITE | ProtFlags::PROT_READ,
        MapFlags::MAP_SHARED,
        fd as usize,
        0,
    );
    assert!(start > 0);

    close(fd as usize);

    // after close ,we still can access the mmap
    let mmap =
        unsafe { core::slice::from_raw_parts_mut(start as *mut u8, stat.st_size as usize + 10) };

    mmap[str.len()] = b'!';

    println!(
        "The content of the file is {}",
        core::str::from_utf8(mmap).unwrap()
    );

    let res = munmap(start as usize, stat.st_size as usize);
    assert_eq!(res, 0);

    println!(
        "The content of the file is {}",
        core::str::from_utf8(mmap).unwrap()
    ); // this will cause a page fault, and be killed by kernel
    println!("Test mmap and munmap passed!")
}
