#![no_std]
#![no_main]

use core::net::{Ipv4Addr, SocketAddr};

use pconst::net::{Domain, SocketAddrIn, SocketType};
use Mstd::{
    println,
    socket::{bind, recvfrom, sendto, socket},
};

#[no_mangle]
fn main() {
    println!("Test socket");
    let server = socket(Domain::AF_INET, SocketType::SOCK_DGRAM, 0);
    if server < 0 {
        println!("Failed to create socket");
        return;
    } else {
        println!("Created socket: {}", server);
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
        return;
    } else {
        println!("socket bind successes!");
    }

    let client = socket(Domain::AF_INET, SocketType::SOCK_DGRAM, 0);
    if client < 0 {
        println!("Failed to create socket");
        return;
    } else {
        println!("Created socket: {}", client);
    }
    let client_addr = SocketAddrIn {
        family: Domain::AF_INET as u16,
        in_port: 20001,
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
        return;
    } else {
        println!("socket bind successes!");
    }

    let data = b"Hello Server!";
    let res = sendto(
        client as usize,
        data.as_ptr(),
        data.len(),
        0,
        &server_addr,
        core::mem::size_of::<SocketAddrIn>(),
    );

    if res != data.len() as isize {
        println!("Failed to send data to server");
        return;
    } else {
        println!("Sent data to server: {:?}", core::str::from_utf8(data));
    }

    let mut buf = [0u8; 512];
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

    let socket_addr = SocketAddr::from(remote_addr);
    println!("receive {} bytes from client {:?}", res, socket_addr);

    if res != data.len() as isize {
        println!("Failed to receive data from client");
        return;
    } else {
        assert_eq!(data, &buf[..res as usize]);
        println!(
            "Received data from client: {:?}",
            core::str::from_utf8(&buf[..res as usize])
        );
    }
    println!("Test socket successes!");
}
