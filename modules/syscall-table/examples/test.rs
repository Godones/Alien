use syscall_table::{register_syscall, Table};
use systable_macro_derive::syscall_func;

fn main() {
    let mut table = Table::new();
    register_syscall!(table, (0, read), (1, test),);
    table.do_call(0, &[1, 2, 0, 0, 0, 0]);
    let data = [6usize; 8];
    table.do_call(1, &[0, 8 * 8, data.as_ptr() as usize, 0, 0, 0]);
    table.register(2, write);
}

#[syscall_func(1)]
fn write() -> isize {
    -1
}

fn read(p1: usize, p2: usize) -> isize {
    println!("p1+p2 = {}", p1 + p2);
    0
}

fn test(p1: usize, p2: usize, p3: *const u8) -> isize {
    let len = p1 + p2;
    let buf = unsafe { core::slice::from_raw_parts(p3, len) };
    // transfer to usize
    let buf = buf
        .chunks(8)
        .map(|x| {
            let mut buf = [0u8; 8];
            buf.copy_from_slice(x);
            usize::from_le_bytes(buf)
        })
        .collect::<Vec<usize>>();
    println!("read {}, buf = {:?}", len, buf);
    0
}
