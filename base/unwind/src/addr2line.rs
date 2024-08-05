use alloc::{borrow::Cow, sync::Arc};
use core::ops::Deref;

use addr2line::{
    gimli::{EndianArcSlice, RunTimeEndian},
    Context,
};
use preprint::pprint as print;
use spin::Lazy;

static CONTEXT: Lazy<SyncContext> = Lazy::new(|| {
    // let data = unsafe {
    //     core::slice::from_raw_parts(&ELF_IMAGE as *const _ as *const u8, 0x40000000 as usize)
    // };
    // let elf = File::parse(data).expect("failed to parse elf image file");
    // let endian = if elf.is_little_endian() {
    //     addr2line::gimli::RunTimeEndian::Little
    // } else {
    //     addr2line::gimli::RunTimeEndian::Big
    // };
    // fn load_section<'data: 'file, 'file, O, Endian>(
    //     id: addr2line::gimli::SectionId,
    //     file: &'file O,
    //     endian: Endian,
    // ) -> Result<addr2line::gimli::read::EndianArcSlice<Endian>, addr2line::gimli::Error>
    // where
    //     O: object::Object<'data, 'file>,
    //     Endian: addr2line::gimli::Endianity,
    // {
    //     use object::ObjectSection;
    //     let data = file
    //         .section_by_name(id.name())
    //         .and_then(|section| section.uncompressed_data().ok())
    //         .unwrap_or_else(||{
    //             Cow::Borrowed(&[])
    //         });
    //     // preprint::pprintln!("load section: {}, len: {}", id.name(), data.len());
    //     Ok(addr2line::gimli::EndianArcSlice::new(
    //         Arc::from(&*data),
    //         endian,
    //     ))
    // }
    /// Load debug info from elf file.
    fn load_section(
        _id: addr2line::gimli::SectionId,
    ) -> Result<EndianArcSlice<RunTimeEndian>, addr2line::gimli::Error> {
        let data = Cow::from(&[]);
        Ok(EndianArcSlice::new(
            Arc::from(&*data),
            RunTimeEndian::Little,
        ))
    }
    let dwarf =
        addr2line::gimli::Dwarf::load(|id| load_section(id)).expect("failed to load dwarf data");
    let ctx = Context::from_dwarf(dwarf).expect("No dwarf data found");
    SyncContext(ctx)
});

pub struct SyncContext(Context<EndianArcSlice<RunTimeEndian>>);
unsafe impl Sync for SyncContext {}
unsafe impl Send for SyncContext {}

impl Deref for SyncContext {
    type Target = Context<EndianArcSlice<RunTimeEndian>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[allow(unused)]
pub fn print_addr2line(addr: u64) {
    print!("addr {:#x}, at ", addr);
    match CONTEXT.find_location(addr).unwrap() {
        Some(loc) => {
            if let Some(ref file) = loc.file.as_ref() {
                print!("{}:", file);
            } else {
                print!("??:");
            }
            print!("{}:{}\n", loc.line.unwrap_or(0), loc.column.unwrap_or(0));
        }
        None => {
            print!("??:0:0\n");
        }
    }
}

impl core::fmt::Display for super::StackFrame {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        let ctx = &CONTEXT;

        write!(f, "function addr {:#x}, at ", self.initial_address)?;
        match ctx.find_location(self.initial_address).unwrap() {
            Some(loc) => {
                if let Some(ref file) = loc.file.as_ref() {
                    write!(f, "{}:", file)?;
                } else {
                    write!(f, "??:")?;
                }
                write!(f, "{}:{}", loc.line.unwrap_or(0), loc.column.unwrap_or(0))?;
                writeln!(f, "")?;
            }
            None => {
                writeln!(f, "??:0:0")?;
            }
        }

        write!(f, "call site addr {:#x}, at ", self.call_site_address)?;
        match ctx.find_location(self.call_site_address).unwrap() {
            Some(loc) => {
                if let Some(ref file) = loc.file.as_ref() {
                    write!(f, "{}:", file)?;
                } else {
                    write!(f, "??:")?;
                }
                write!(f, "{}:{}", loc.line.unwrap_or(0), loc.column.unwrap_or(0))?;
                writeln!(f, "")?;
            }
            None => {
                writeln!(f, "??:0:0")?;
            }
        }
        Ok(())
    }
}
