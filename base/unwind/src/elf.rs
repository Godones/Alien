use core::ops::Range;

use gimli::BaseAddresses;
use spin::{Lazy, Once};

extern "C" {
    fn kernel_eh_frame_hdr();
    fn kernel_eh_frame_hdr_end();
    fn kernel_eh_frame();
    fn kernel_eh_frame_end();
    fn kernel_gcc_except_table();
    fn kernel_gcc_except_table_end();
    fn stext();
    fn etext();
}

static BASE_ADDRESSES: Lazy<BaseAddresses> = Lazy::new(|| {
    let eh_frame = section_by_name(".eh_frame").unwrap();
    let eh_frame_hdr = section_by_name(".eh_frame_hdr").unwrap();
    let text = section_by_name(".text").unwrap();
    debug!(
        "eh_frame: range [0x{:016x} - 0x{:016x}]",
        eh_frame.start, eh_frame.end
    );
    debug!(
        "eh_frame_hdr: range [0x{:016x} - 0x{:016x}]",
        eh_frame_hdr.start, eh_frame_hdr.end
    );
    debug!("text: range [0x{:016x} - 0x{:016x}]", text.start, text.end);
    BaseAddresses::default()
        .set_eh_frame_hdr(eh_frame_hdr.start)
        .set_eh_frame(eh_frame.start)
        .set_text(text.start)
});

pub fn base_addresses() -> BaseAddresses {
    BASE_ADDRESSES.clone()
}

static EH_FRAME: Once<Range<u64>> = Once::new();

fn eh_frame() -> Range<u64> {
    EH_FRAME
        .get()
        .unwrap_or_else(|| EH_FRAME.call_once(|| section_by_name(".eh_frame").unwrap()))
        .clone()
}

pub fn eh_frame_slice() -> &'static [u8] {
    let eh_frame = eh_frame();
    unsafe {
        core::slice::from_raw_parts(
            eh_frame.start as usize as *const u8,
            (eh_frame.end - eh_frame.start) as usize,
        )
    }
}

fn section_by_name(name: &'static str) -> Option<Range<u64>> {
    match name {
        ".eh_frame" => Some(Range {
            start: kernel_eh_frame as u64,
            end: kernel_eh_frame_end as u64,
        }),
        ".eh_frame_hdr" => Some(Range {
            start: kernel_eh_frame_hdr as u64,
            end: kernel_eh_frame_hdr_end as u64,
        }),
        ".text" => Some(Range {
            start: stext as u64,
            end: etext as u64,
        }),
        name => {
            panic!("Get section by name {} from ELF file failed!!!", name);
        }
    }
}

pub fn find_lsda_data(addr: usize) -> Option<&'static [u8]> {
    if addr >= kernel_gcc_except_table as usize && addr < kernel_gcc_except_table_end as usize {
        let offset = addr - kernel_gcc_except_table as usize;
        let slice = unsafe {
            core::slice::from_raw_parts(
                kernel_gcc_except_table as *const u8,
                kernel_gcc_except_table_end as usize - kernel_gcc_except_table as usize,
            )
        };
        Some(&slice[offset..])
    } else {
        panic!("Get section by addr {:#x} from ELF file failed!!!", addr);
    }
}
