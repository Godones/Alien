use std::alloc::{alloc, Layout};
use std::sync::Arc;
use uart::Uart;

enum ProcessState{
    Waiting,
    Readying,
}
struct Process{
    state:ProcessState
}

fn test_uart(){
    let addr = unsafe { alloc(Layout::from_size_align(8, 1).unwrap()) };
    let addr = addr as usize;
    let uart = Uart::<Arc<Process>>::new(addr);
    let process = Arc::new(Process{state:ProcessState::Readying});

    let wait = |x:&mut Vec<Arc<Process>>| {
        let process = process.clone();
        x.push(process);
        println!("{}",x.len());
    };

    let schedule = ||{
        println!("schedule ....");
    };
    for _i in 0..128{
        uart.put_ch(1u8,wait ,schedule);
    }
    // uart.put(1u8,wait ,schedule);
}
fn main(){
    test_uart();
}