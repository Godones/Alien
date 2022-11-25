/// 获取当前计时器的值
#[inline]
pub fn read_timer() -> usize {
    riscv::register::time::read()
}

/// 设置下一次时钟的中断
#[inline]
pub fn set_next_trigger(addition: usize) {
    crate::sbi::set_timer(read_timer() + addition)
}
