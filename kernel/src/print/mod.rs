use console::PrePrint;
use logging::init_logger;

#[macro_use]
pub mod console;
mod logging;

pub fn init_print() {
    init_logger();
    preprint::init_print(&PrePrint);
    println!("Print init success");
}
