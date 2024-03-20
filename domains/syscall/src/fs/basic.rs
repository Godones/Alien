use alloc::sync::Arc;
use basic::println;
use constants::AlienResult;
use interface::{TaskDomain, VfsDomain};
use rref::RRefVec;

pub fn sys_write(
    vfs: &Arc<dyn VfsDomain>,
    task_domain: &Arc<dyn TaskDomain>,
    fd: usize,
    buf: *const u8,
    len: usize,
) -> AlienResult<isize> {
    let file = task_domain.get_fd(fd)?;
    let tmp_buf = RRefVec::<u8>::new(0, len);
    task_domain.copy_from_user(buf, tmp_buf.as_slice().as_ptr() as *mut u8, len)?;
    let str = core::str::from_utf8(&tmp_buf.as_slice()).unwrap();
    println!("write: {:?}", str);
    let w = vfs.vfs_write(file, &tmp_buf);
    println!("write: {:?} bytes", w);
    w.map(|x| x as isize)
}
