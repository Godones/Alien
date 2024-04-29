#![no_std]
#![no_main]

use Mstd::{
    domain::{register_domain, update_domain, DomainTypeRaw},
    fs::{open, OpenFlags},
    println,
};

#[no_mangle]
fn main() -> isize {
    let random_scheduler_fd = open("/tests/grandom_scheduler\0", OpenFlags::O_RDONLY);
    if random_scheduler_fd < 0 {
        println!("Failed to open /tests/grandom_scheduler");
        return -1;
    } else {
        println!(
            "Opened /tests/grandom_scheduler, fd: {}",
            random_scheduler_fd
        );
        let res = register_domain(
            random_scheduler_fd as _,
            DomainTypeRaw::SchedulerDomain,
            "random_scheduler",
        );
        println!("load_domain res: {}", res);

        if res != 0 {
            println!("Failed to register domain random_scheduler");
            return -1;
        }
        let res = update_domain(
            "scheduler",
            "random_scheduler",
            DomainTypeRaw::SchedulerDomain,
        );
        println!("replace_domain res: {}", res);
        if res != 0 {
            println!("Failed to update domain random_scheduler");
            return -1;
        }
    }
    println!("Test register and update scheduler domain");
    0
}
