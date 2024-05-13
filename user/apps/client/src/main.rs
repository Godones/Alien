#![no_std]
#![no_main]

extern crate alloc;

use alloc::{format, string::String, vec::Vec};
use core::net::Ipv4Addr;

use pconst::net::{Domain, SocketAddrIn, SocketType};
use Mstd::{
    fs::{open, write, OpenFlags},
    println,
    socket::{bind, recvfrom, sendto, socket},
};

const BEGIN: &str = "BEGIN:I'm qemu client, begin to send data";
const END: &str = "END:I'm qemu client, end to send data";

#[no_mangle]
fn main(_: usize, argv: Vec<String>) -> isize {
    if argv.len() != 2 {
        println!("Usage: ./client <string>");
        return 0;
    }
    println!("I'm qemu client, try to receive domain data from server");
    let domain_name = &argv[1];
    let client = socket(Domain::AF_INET, SocketType::SOCK_DGRAM, 0);
    if client < 0 {
        println!("Failed to create socket");
        return -1;
    }
    let port = 2000u16;
    let client_addr = SocketAddrIn {
        family: Domain::AF_INET as u16,
        in_port: port.to_be(),
        addr: Ipv4Addr::new(10, 0, 2, 15),
        sin_zero: [0; 8],
    };

    let res = bind(
        client as usize,
        &client_addr,
        core::mem::size_of::<SocketAddrIn>(),
    );
    if res < 0 {
        println!("Failed to bind socket");
        return -1;
    }
    println!("qemu client socket bind successes!");

    let mut buf = [0u8; 1024];
    let port = 50000u16;
    let server_addr = SocketAddrIn {
        family: Domain::AF_INET as u16,
        in_port: port.to_be(),
        addr: Ipv4Addr::new(10, 0, 2, 2),
        sin_zero: [0; 8],
    };

    // send began
    let send = sendto(
        client as usize,
        BEGIN.as_bytes().as_ptr(),
        BEGIN.len(),
        0,
        &server_addr as *const SocketAddrIn,
        core::mem::size_of::<SocketAddrIn>(),
    );
    if send != BEGIN.len() as isize {
        println!("Failed to send data to server");
        return -1;
    }

    println!("Send BEGIN [{} bytes] to server successes", BEGIN.len());
    let mut remote_addr = SocketAddrIn::default();
    let mut size = 0usize;

    println!("Try receive reply");
    let res = recvfrom(
        client as usize,
        buf.as_mut_ptr(),
        buf.len(),
        0,
        &mut remote_addr as *mut SocketAddrIn,
        &mut size as *mut usize,
    );
    if res < 0 {
        println!("Failed to receive data from server");
        return -1;
    }
    println!(
        "Received data from server: {:?}",
        core::str::from_utf8(&buf[..res as usize])
    );
    // make sure the data is "BEGIN:OK"
    if core::str::from_utf8(&buf[..res as usize]) != Ok("BEGIN:OK") {
        println!("Received data is not correct");
        println!("Server should send \"BEGIN:OK\" to client");
        return -1;
    }

    let domain_req = format!("GET:{}", domain_name);
    let send = sendto(
        client as usize,
        domain_req.as_bytes().as_ptr(),
        domain_req.len(),
        0,
        &server_addr as *const SocketAddrIn,
        core::mem::size_of::<SocketAddrIn>(),
    );
    if send != domain_req.as_bytes().len() as isize {
        println!("<GET:Domain> Failed to send data to server");
        return -1;
    }
    let res = recvfrom(
        client as usize,
        buf.as_mut_ptr(),
        buf.len(),
        0,
        &mut remote_addr as *mut SocketAddrIn,
        &mut size as *mut usize,
    );
    if res < 0 {
        println!("Failed to receive data from server");
        return -1;
    }
    println!(
        "Received data from server: {:?}",
        core::str::from_utf8(&buf[..res as usize])
    );
    // make sure the data is "BEGIN:OK"
    if core::str::from_utf8(&buf[..res as usize]) != Ok("GET:OK") {
        println!("Received data is not correct");
        println!("Server should send \"GET:OK\" to client");
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

    let mut count = 0usize;

    loop {
        let data_req = format!("DATA:{}", count);
        let send = sendto(
            client as usize,
            data_req.as_bytes().as_ptr(),
            data_req.len(),
            0,
            &server_addr as *const SocketAddrIn,
            core::mem::size_of::<SocketAddrIn>(),
        );
        assert_eq!(send, data_req.as_bytes().len() as isize);
        let res = recvfrom(
            client as usize,
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
        println!("Received {} bytes data from client", res);
        if res == 6 && core::str::from_utf8(&buf[..res as usize]) == Ok("NODATA") {
            println!("No more data to receive");
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
        count += res as usize;
    }
    sendto(
        client as usize,
        END.as_bytes().as_ptr(),
        END.len(),
        0,
        &server_addr as *const SocketAddrIn,
        core::mem::size_of::<SocketAddrIn>(),
    );

    println!(
        "Received {} bytes data from client, and write to file successes",
        count
    );
    0
}
