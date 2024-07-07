#![no_std]
#![no_main]

extern crate alloc;

use alloc::{string::String, vec::Vec};

use domain_helper::DomainHelperBuilder;
use Mstd::{
    domain::DomainTypeRaw,
    fs::{close, open, OpenFlags},
    println,
};

#[no_mangle]
fn main(_: usize, argv: Vec<String>) -> isize {
    if argv.len() != 2 {
        println!("Usage: dlog [new]/[old]");
        return -1;
    }
    let option = argv[1].as_str();
    match option {
        "new" => {
            println!("Register and attach xlogger domain");
            let xlogger_fd = open("/tests/domains/gxlogger\0", OpenFlags::O_RDONLY);
            if xlogger_fd < 0 {
                println!("Failed to open /tests/domains/gxlogger");
                let res = downloader::download_domain("gxlogger", "/tests/domains");
                match res {
                    Err(e) => {
                        println!("Failed to download domain: {}", e);
                        return -1;
                    }
                    Ok(_) => {
                        println!("Download domain gxlogger successfully");
                    }
                }
            } else {
                close(xlogger_fd as _);
            }
            let builder = DomainHelperBuilder::new()
                .ty(DomainTypeRaw::LogDomain)
                .domain_name("logger")
                .domain_file_name("xlogger")
                .domain_file_path("/tests/domains/gxlogger\0");
            builder.clone().register_domain_file().unwrap();
            builder.clone().update_domain().unwrap();
            println!("Register and update logger domain to new version successfully");
        }
        "old" => {
            DomainHelperBuilder::new()
                .ty(DomainTypeRaw::LogDomain)
                .domain_name("logger")
                .domain_file_name("logger")
                .update_domain()
                .unwrap();
            println!("Update logger domain to old version successfully");
        }
        _ => {
            println!("Usage: dlog [new]/[old]");
            return -1;
        }
    }
    0
}
