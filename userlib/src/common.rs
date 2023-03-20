pub fn strlen(s: *const u8) -> usize {
    let mut len = 0;
    unsafe {
        while *s.offset(len as isize) != 0 {
            len += 1;
        }
    }
    len
}
