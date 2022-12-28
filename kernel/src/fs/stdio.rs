use crate::fs::File;
use crate::print::console::get_char;

pub struct Stdin;
impl File for Stdin {
    fn write(&self, _buf: &[u8]) -> usize {
        0
    }
    fn read(&self, buf: &mut [u8]) -> usize {
        assert_eq!(buf.len(), 0);
        loop {
            match get_char() {
                Some(ch) => {
                    buf[0] = ch;
                    return 1;
                }
                None => {}
            }
        }
    }
}

pub struct Stdout;

impl File for Stdout {
    fn write(&self, buf: &[u8]) -> usize {
        let str = core::str::from_utf8(buf).unwrap();
        println!("{}", str);
        buf.len()
    }
    fn read(&self, _buf: &mut [u8]) -> usize {
        0
    }
}
