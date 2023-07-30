#[repr(align(4))]
struct _Wrapper<T>(T);

pub const FDT: &[u8] = &_Wrapper(*include_bytes!("../../../tools/hifive-unmatched-a00.dtb")).0;
