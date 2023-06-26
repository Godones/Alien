use Mstd::fs::{close, fstatat, fstatfs, open, OpenFlags, Stat, StatFlags, StatFs, statfs};

pub fn stat_test() -> isize {
    let fd = open("/db/stattest\0", OpenFlags::O_CREAT | OpenFlags::O_WRONLY);
    if fd == -1 {
        println!("open error");
    }
    println!("open file /db/stattest fd = {}", fd);
    let mut stat_fs = StatFs::default();
    let res = fstatfs(fd as usize, &mut stat_fs);
    if res == -1 {
        println!("fstatfs error");
    }
    println!("fstatfs:{:#?}", stat_fs);

    let res = statfs("/db/stattest\0", &mut stat_fs);
    if res == -1 {
        println!("statfs error");
    }
    println!("statfs:{:#?}", stat_fs);

    let mut stat_file = Stat::default();
    let res = fstatat(fd, "/db/stattest\0", &mut stat_file, StatFlags::empty());
    if res == -1 {
        println!("fstatat error");
    }
    println!("fstatat:{:#?}", stat_file);
    close(fd as usize);
    0
}