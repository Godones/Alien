#![no_std]
#![no_main]
extern crate alloc;

use alloc::{string::String, vec::Vec};

use domain_helper::DomainHelperBuilder;
use Mstd::{domain::DomainTypeRaw, println};

#[no_mangle]
fn main(_: usize, argv: Vec<String>) -> isize {
    if argv.len() != 2 {
        println!("Usage: dtask [new]/[old]");
        return -1;
    }
    let option = argv[1].as_str();
    match option {
        "old" => {
            let builder = DomainHelperBuilder::new()
                .domain_name("scheduler")
                .ty(DomainTypeRaw::SchedulerDomain)
                .domain_file_name("fifo_scheduler");
            builder.update_domain().unwrap();
            println!("Update scheduler domain to old version success");
        }
        "new" => {
            let builder = DomainHelperBuilder::new()
                .domain_name("scheduler")
                .ty(DomainTypeRaw::SchedulerDomain)
                .domain_file_path("/tests/grandom_scheduler\0")
                .domain_file_name("prio_scheduler");
            builder.clone().register_domain_file().unwrap();
            builder.update_domain().unwrap();
            println!("Update scheduler domain to new version success");
        }
        _ => {
            println!("Usage: dtask [new]/[old]");
            return -1;
        }
    }
    0
}
