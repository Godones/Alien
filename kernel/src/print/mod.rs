#[macro_use]
pub mod console;
mod logging;

pub use console::PrePrint;
pub use logging::init_logger;
