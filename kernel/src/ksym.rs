use alloc::{ vec::Vec};

use ksym::KallsymsMapped;
use spin::Once;
use unwinder::PanicHelper;

use crate::fs::read_all;

pub static KALLSYMS: Once<KallsymsMapped<'static>> = Once::new();

extern "C" {
    fn stext();
    fn srodata();
}

pub fn init_kallsyms() {
    let stext = stext as usize;
    let etext = srodata as usize;

    let mut buf = Vec::new();
    read_all("/tests/kallsyms", &mut buf);
    let buf = buf.leak();

    let ksym = ksym::KallsymsMapped::from_blob(buf, stext as u64, etext as u64)
        .expect("Failed to map kallsyms");
    KALLSYMS.call_once(|| ksym);
}


pub struct KernelPanicHelper;
impl PanicHelper for KernelPanicHelper {
    fn lookup_symbol<'a>(&self, addr: usize, buf: &'a mut [u8; 1024]) -> Option<(&'a str, usize)> {
        let ksym = KALLSYMS.get()?;
        ksym.lookup_address(addr as _, buf)
            .map(|(name, _size, offset, _ty)| (name, addr - offset as usize))
    }
}
