pub const CLOCKS_PER_SEC: usize = 12500000;

pub const FRAME_SIZE: usize = 4096;

pub fn strlen(s: *const u8) -> usize {
    let mut len = 0;
    unsafe {
        while *s.offset(len as isize) != 0 {
            len += 1;
        }
    }
    len
}
