#![no_std]
#![no_main]

extern crate alloc;

use alloc::{string::String, vec::Vec};
use core::net::Ipv4Addr;

use pconst::net::{Domain, SocketAddrIn, SocketType};
use Mstd::{
    fs::{open, write, OpenFlags},
    println,
    socket::{bind, recvfrom, socket},
};

#[no_mangle]
fn main(_: usize, argv: Vec<String>) -> isize {
    if argv.len() != 2 {
        println!("Usage: ./server <string>");
        return 0;
    }
    let domain_name = &argv[1];
    let server = socket(Domain::AF_INET, SocketType::SOCK_DGRAM, 0);
    if server < 0 {
        println!("Failed to create socket");
        return -1;
    }
    let server_addr = SocketAddrIn {
        family: Domain::AF_INET as u16,
        in_port: 2000,
        addr: Ipv4Addr::new(10, 0, 2, 15),
        sin_zero: [0; 8],
    };

    let res = bind(
        server as usize,
        &server_addr,
        core::mem::size_of::<SocketAddrIn>(),
    );
    if res < 0 {
        println!("Failed to bind socket");
        return -1;
    }
    println!("socket bind successes!");

    // began to receive data

    let mut buf = [0u8; 1024];
    let mut remote_addr = SocketAddrIn::default();
    let mut size = 0usize;

    let res = recvfrom(
        server as usize,
        buf.as_mut_ptr(),
        buf.len(),
        0,
        &mut remote_addr as *mut SocketAddrIn,
        &mut size as *mut usize,
    );
    if res < 0 {
        println!("Failed to receive data from client");
        return -1;
    }
    println!(
        "Received data from server: {:?}",
        core::str::from_utf8(&buf[..res as usize])
    );
    // make sure the data is "Hello Server!"
    if core::str::from_utf8(&buf[..res as usize]) != Ok("Hello Server!") {
        println!("Received data is not correct");
        println!("Client should send \"Hello Server!\" to server");
        return -1;
    }

    // open file to store the data
    let mut domain_name = domain_name.clone();
    domain_name.push('\0');
    let file = open(&domain_name, OpenFlags::O_CREAT | OpenFlags::O_RDWR);
    if file < 0 {
        println!("Failed to open file");
        return -1;
    }

    let mut count = 0;

    loop {
        let res = recvfrom(
            server as usize,
            buf.as_mut_ptr(),
            buf.len(),
            0,
            &mut remote_addr as *mut SocketAddrIn,
            &mut size as *mut usize,
        );
        if res < 0 {
            println!("Failed to receive data from client");
            return -1;
        }
        if res == 0 {
            break;
        }
        let w = write(file as usize, &buf[..res as usize]);
        if w < 0 || w != res {
            println!(
                "Failed to write data to file, write_len: {}, res: {}",
                res, w
            );
            return -1;
        }
        count += res;
    }

    println!(
        "Received {} bytes data from client, and write to file successes",
        count
    );
    0
}
