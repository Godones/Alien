#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::boxed::Box;

use basic::{println, AlienResult};
use interface::{Basic, Level, LevelFilter, LogDomain};
use log::{Log, Metadata, Record};
use rref::RRefVec;

#[derive(Debug)]
pub struct Logger;

impl Basic for Logger {}

impl LogDomain for Logger {
    fn init(&self) -> AlienResult<()> {
        log::set_logger(&SimpleLogger).unwrap();
        // default log level
        log::set_max_level(log::LevelFilter::Trace);
        println!("Logger init");
        Ok(())
    }

    fn log(&self, level: Level, msg: RRefVec<u8>) -> AlienResult<()> {
        let msg = core::str::from_utf8(msg.as_slice()).unwrap();
        let level = match level {
            Level::Error => log::Level::Error,
            Level::Warn => log::Level::Warn,
            Level::Info => log::Level::Info,
            Level::Debug => log::Level::Debug,
            Level::Trace => log::Level::Trace,
        };
        log::log!(level, "{}", msg);
        Ok(())
    }

    fn set_max_level(&self, level: LevelFilter) -> AlienResult<()> {
        log::set_max_level(match level {
            LevelFilter::Error => log::LevelFilter::Error,
            LevelFilter::Warn => log::LevelFilter::Warn,
            LevelFilter::Info => log::LevelFilter::Info,
            LevelFilter::Debug => log::LevelFilter::Debug,
            LevelFilter::Trace => log::LevelFilter::Trace,
            _ => log::LevelFilter::Off,
        });
        println!("Logger set_max_level: {:?}", level);
        Ok(())
    }
}

struct SimpleLogger;

impl Log for SimpleLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }
    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let color = match record.level() {
            log::Level::Error => 31, // Red
            log::Level::Warn => 93,  // BrightYellow
            log::Level::Info => 35,  // Blue
            log::Level::Debug => 32, // Green
            log::Level::Trace => 90, // BrightBlack
        };
        println!(
            "\u{1B}[{}m[{:>1}] {}\u{1B}[0m",
            color,
            record.level(),
            record.args(),
        );
    }
    fn flush(&self) {}
}

pub fn main() -> Box<dyn LogDomain> {
    Box::new(Logger)
}
