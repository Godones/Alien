#![no_std]
#![no_main]

extern crate Mstd;

use core::str;
use Mstd::fs::close;
use Mstd::println;
use Mstd::process::{exit, fork};
use Mstd::socket::*;
use Mstd::time::sleep;

const INTADDR: u32 = 2130706433; //127.0.0.1
const PORT: u16 = 8899;
const BUF_SIZE: usize = 1024;

#[no_mangle]
fn main() {
    println!("socket_tests...");
    tcp_test();
    udp_test();
    println!("socket_tests passed!");
}

fn tcp_test() {
    println!("tcp_test...");
    let pid = fork();
    assert!(pid >= 0);
    if pid != 0 {
        // parent for server
        // socket() -> bind() -> listen() -> accept() -> recv() -> send() -> close()
        let sockfd = socket(Domain::AF_INET, SocketType::SOCK_STREAM, 0);
        assert!(sockfd > 0);
        let addr = Sockaddr::new(Domain::AF_INET, INTADDR, PORT);
        assert!(
            bind(
                sockfd as usize,
                &addr as *const Sockaddr,
                core::mem::size_of::<Sockaddr>()
            ) >= 0
        );
        assert!(listen(sockfd as usize, 10) >= 0);
        println!("tcp server listening...");
        let mut peerlen: usize = 0;
        let mut peeraddr: Sockaddr = addr.clone();
        loop {
            let connfd: isize = accept(
                sockfd as usize,
                &mut peeraddr as *mut Sockaddr,
                &mut peerlen as *mut usize,
            );
            assert!(connfd >= 0);
            let mut buf = [0u8; BUF_SIZE];
            let recvnum = recv(connfd as usize, &mut buf as *mut u8, BUF_SIZE, 0);
            assert!(recvnum >= 0);
            println!(
                "tcp server receive a message: {}",
                str::from_utf8(&buf).unwrap()
            );

            let msg: &[u8] = b"Hi, I'm Alien's server";
            send(connfd as usize, msg.as_ptr() as *const u8, msg.len(), 0);
            close(connfd as usize);
            break;
        }
        close(sockfd as usize);
        // sleeping for client receive the msg
        sleep(1000);
    } else {
        // child for client,  sleeping for sevrer start
        // socket() -> connect() -> send() -> recv() -> close()
        sleep(1000);
        let sockfd = socket(Domain::AF_INET, SocketType::SOCK_DGRAM, 0);
        assert!(sockfd > 0);
        let addr = Sockaddr::new(Domain::AF_INET, INTADDR, PORT);
        let buf: &[u8] = b"Hello, this is Alien's client";
        assert!(
            connect(
                sockfd as usize,
                &addr as *const Sockaddr,
                core::mem::size_of::<Sockaddr>()
            ) >= 0
        );
        send(sockfd as usize, buf.as_ptr() as *const u8, buf.len(), 0);
        let recvnum = recv(sockfd as usize, buf.as_ptr() as *mut u8, BUF_SIZE, 0);
        assert!(recvnum >= 0);
        println!(
            "tcp client receive a message: {}",
            str::from_utf8(&buf).unwrap()
        );
        println!("tcp test passed!");
        close(sockfd as usize);
        exit(0);
    }
}

fn udp_test() {
    println!("udp_test...");
    let pid = fork();
    assert!(pid >= 0);
    if pid != 0 {
        // parent for server
        // socket() -> bind() -> recvfrom() -> close()
        let sockfd = socket(Domain::AF_INET, SocketType::SOCK_DGRAM, 0);
        assert!(sockfd > 0);
        let addr = Sockaddr::new(Domain::AF_INET, INTADDR, PORT);
        assert!(
            bind(
                sockfd as usize,
                &addr as *const Sockaddr,
                core::mem::size_of::<Sockaddr>()
            ) >= 0
        );
        loop {
            let mut buf = [0u8; BUF_SIZE];
            let mut peerlen: usize = 0;
            let mut peeraddr: Sockaddr = addr.clone();
            let recvnum = recvfrom(
                sockfd as usize,
                &mut buf as *mut u8,
                BUF_SIZE,
                0,
                &mut peeraddr as *mut Sockaddr,
                &mut peerlen as *mut usize,
            );
            assert!(recvnum >= 0);
            println!(
                "udp server receive a message: {}",
                str::from_utf8(&buf).unwrap()
            );
            break;
        }
        close(sockfd as usize);
        println!("udp test passed!");
    } else {
        // child for client,  sleeping for sevrer start
        // socket() -> sendto() -> close()
        sleep(1000);
        let sockfd = socket(Domain::AF_INET, SocketType::SOCK_STREAM, 0);
        assert!(sockfd > 0);
        let addr = Sockaddr::new(Domain::AF_INET, INTADDR, PORT);
        let buf: &[u8] = b"Hello, this is Alien's client";
        sendto(
            sockfd as usize,
            buf.as_ptr() as *const u8,
            buf.len(),
            0,
            &addr as *const Sockaddr,
            core::mem::size_of::<Sockaddr>(),
        );
        println!("udp client send a msg");

        close(sockfd as usize);
        exit(0);
    }
}
