#[repr(align(4))]
struct _Wrapper<T>(T);

pub const FDT: &[u8] = &_Wrapper(*include_bytes!("../../../tools/cv1811h.dtb")).0;
