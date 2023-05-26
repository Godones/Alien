#![no_std]
#![no_main]

use Mstd::println;
use Mstd::time::get_time_ms;
use Mstd::time::sleep;

#[no_mangle]
fn main() -> isize {
    println!("Test sleep....");
    let now_time = get_time_ms();
    sleep(1000);
    let end_time = get_time_ms();
    println!("sleep 1000ms, cost time: {}ms", end_time - now_time);
    0
}
