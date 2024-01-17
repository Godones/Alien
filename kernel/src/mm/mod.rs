use arch::hart_id;

pub mod elf;
pub mod loader;
pub mod map;

/// This function will be call in slab allocator
#[no_mangle]
fn current_cpu_id() -> usize {
    hart_id()
}

/// (待实现)在一组线程中，设置内存屏障，控制多核系统中的内存访问次序。目前直接返回0。
///
///<https://man7.org/linux/man-pages/man2/membarrier.2.html>
#[syscall_func(283)]
pub fn membarrier() -> isize {
    0
}
