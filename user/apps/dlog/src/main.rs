#![no_std]
#![no_main]

extern crate alloc;

use alloc::{string::String, vec::Vec};

use Mstd::{
    domain::{register_domain, update_domain, DomainTypeRaw},
    fs::{open, OpenFlags},
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
            let is_exist = open("/tests/domains/gxlogger\0", OpenFlags::O_RDONLY);
            if is_exist < 0 {
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
            }
            let xlogger_fd = open("/tests/domains/gxlogger\0", OpenFlags::O_RDONLY);
            if xlogger_fd < 0 {
                println!("Failed to open /tests/gxlogger");
                return -1;
            } else {
                println!("Opened /tests/domains/gxlogger, fd: {}", xlogger_fd);
                let res = register_domain(xlogger_fd as _, DomainTypeRaw::LogDomain, "xlogger");
                println!("register_domain res: {}", res);

                if res != 0 {
                    println!("Failed to register domain xlogger");
                    return -1;
                }
                let res = update_domain("logger", "xlogger", DomainTypeRaw::LogDomain);
                println!("replace_domain res: {}", res);
                if res != 0 {
                    println!("Failed to update domain xlogger");
                    return -1;
                }
            }
        }
        "old" => {
            let res = update_domain("logger", "logger", DomainTypeRaw::LogDomain);
            println!("replace_domain res: {}", res);
            if res != 0 {
                println!("Failed to update domain logger");
                return -1;
            }
        }
        _ => {
            println!("Usage: dlog [new]/[old]");
            return -1;
        }
    }
    println!("Test register and update log domain");
    0
}
