use crate::syscall::sys_brk;

pub fn brk(addr: usize) -> isize {
    sys_brk(addr)
}

pub fn sbrk(incr: isize) -> isize {
    let addr = brk(0);
    if incr == 0 {
        return addr;
    }
    let new_addr = brk((addr + incr) as usize);
    if new_addr == addr + incr {
        return addr;
    }
    -1
}
