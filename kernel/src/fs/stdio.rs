use crate::fs::{File, UserBuffer};
use crate::print::console::get_char;
#[derive(Debug)]
pub struct Stdin;

impl File for Stdin {
    fn write(&self, _buf: UserBuffer) -> usize {
        0
    }
    fn read(&self, mut buf: UserBuffer) -> usize {
        assert_eq!(buf.len(), 1);
        loop {
            match get_char() {
                Some(ch) => {
                    let buf0 = buf[0].as_mut();
                    buf0[0] = ch as u8;
                    return 1;
                }
                None => {}
            }
        }
    }
}

#[derive(Debug)]
pub struct Stdout;

impl File for Stdout {
    fn write(&self, buf: UserBuffer) -> usize {
        buf.iter().for_each(|buf| {
            let str = core::str::from_utf8(buf).unwrap();
            print!("{}", str);
        });
        buf.iter().map(|buf| buf.len()).sum()
    }
    fn read(&self, _buf: UserBuffer) -> usize {
        0
    }
}
