#[repr(align(4))]
struct _Wrapper<T>(T);

pub const FDT: &[u8] = &_Wrapper(*include_bytes!("../../../tools/jh7110-visionfive-v2.dtb")).0;
