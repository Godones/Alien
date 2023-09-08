use core::arch::global_asm;


extern "C" {
    #[allow(unused)]
    fn symbol_num();
    #[allow(unused)]
    fn symbol_address();
    #[allow(unused)]
    fn symbol_index();
    #[allow(unused)]
    fn symbol_name();
}

global_asm!(include_str!("kernel_symbol.S"));

pub fn find_symbol_with_addr(addr:usize)->Option<(usize,&'static str)>{
    let symbol_num_addr = symbol_num as usize as *const usize;
    let symbol_num = unsafe { symbol_num_addr.read_volatile() };
    if symbol_num == 0 {
        return None;
    }
    let symbol_addr = symbol_address as usize as *const usize; // 符号地址存储区域
    let addr_data = unsafe { core::slice::from_raw_parts(symbol_addr, symbol_num) };
    // find the symbol with the nearest address
    let mut index = -1isize;
    for i in 0..symbol_num-1{
        if addr>= addr_data[i] && addr<addr_data[i+1]{
            index = i as isize;
            break;
        }
    }
    if addr == addr_data[symbol_num-1]{
        index = (symbol_num-1) as isize;
    }
    if index == -1{
       return None;
    }
    let index = index as usize;
    let symbol_index = symbol_index as usize as *const usize; // 符号字符串的起始位置
    let index_data = unsafe { core::slice::from_raw_parts(symbol_index, symbol_num) };
    let symbol_name = symbol_name as usize as *const u8; // 符号字符串
    let mut last = 0;
    unsafe {
        for i in index_data[index].. {
            let c = symbol_name.add(i);
            if *c == 0 {
                last = i;
                break;
            }
        }
    }
    let name = unsafe {
        core::slice::from_raw_parts(
            symbol_name.add(index_data[index]),
            last - index_data[index],
        )
    };
    let name = core::str::from_utf8(name).unwrap();
    Some((addr_data[index], name))
}

