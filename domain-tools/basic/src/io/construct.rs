use memory_addr::VirtAddr;

/// # Safety
/// caller must ensure that memory area is available during return value's lifetime
pub fn construct_ref_mut<'a, T>(va: VirtAddr) -> &'a mut T {
    unsafe { &mut *((va.as_usize()) as *mut T) }
}
