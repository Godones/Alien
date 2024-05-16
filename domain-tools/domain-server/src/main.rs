use std::{cmp::min, fs::OpenOptions, io::Read, net::UdpSocket, path::Path};

fn main() {
    println!("This is domain-server!");
    // qemu port
    let sock = UdpSocket::bind("127.0.0.1:50000").expect("bind failed");
    let mut buf = [0; 1024];
    // let qemu = SocketAddrV4::new(Ipv4Addr::new(127,0,0,1), 5555);
    let mut file = loop {
        let (amt, src) = sock.recv_from(&mut buf).expect("recv_from failed");
        println!("Received data from client {:?}", src);
        let req = Request::parse(&buf[..amt]);
        match req {
            Request::Begin(req) => {
                println!("Qemu client send req: {}", req);
                sock.send_to("BEGIN:OK".as_bytes(), src)
                    .expect("send failed");
            }
            Request::Get(domain) => {
                println!("Qemu client try to get file: {}", domain);
                let path = Path::new("./build/disk/");
                let path = path.join(format!("{}", domain));
                println!("Path: {:?}, exist: {}", path, path.exists());
                let file = OpenOptions::new().read(true).open(&path);
                match file {
                    Ok(f) => {
                        sock.send_to("GET:OK".as_bytes(), src).expect("send failed");
                        break f;
                    }
                    Err(_) => {
                        sock.send_to("GET:ERR".as_bytes(), src)
                            .expect("send failed");
                    }
                }
            }
            Request::Other(req) => {
                println!("Other request: {}", req);
                println!("Client should send request correctly");
                return;
            }
            _ => {
                println!("Client should send request correctly");
                return;
            }
        }
    };
    let mut data = Vec::new();
    let file_len = file.read_to_end(&mut data).unwrap();
    loop {
        let (amt, src) = sock.recv_from(&mut buf).expect("recv_from failed");
        if amt == 0 {
            panic!("Client should send request correctly");
        }
        let req = Request::parse(&buf[..amt]);
        match req {
            Request::Data(offset) => {
                let end = min(offset + 512, file_len);
                // send 512 bytes data
                if offset >= file_len {
                    sock.send_to("NODATA".as_bytes(), src).expect("send failed");
                    continue;
                }
                // println!("Qemu client send req: {offset}");
                sock.send_to(&data[offset..end], src).expect("send failed");
            }
            Request::End(end) => {
                println!("Qemu client send req: {end}");
                break;
            }
            _ => {
                println!("Client should send request correctly");
                return;
            }
        }
    }
}

pub enum Request {
    // GET:{DomainName}
    Get(String),
    // BEGIN:I'm qemu client, begin to send data
    Begin(String),
    // END:I'm qemu client, end to send data
    End(String),
    // DATA:{offset}
    Data(usize),
    Other(String),
}

const BEGIN: &str = "BEGIN:I'm qemu client, begin to send data";
const END: &str = "END:I'm qemu client, end to send data";

impl Request {
    pub fn parse(buf: &[u8]) -> Request {
        let req = String::from_utf8_lossy(buf);
        let req = req.trim();
        match req {
            BEGIN => Request::Begin(req.to_string()),
            END => Request::End(req.to_string()),
            _ => {
                let mut iter = req.splitn(2, ':');
                match (iter.next(), iter.next()) {
                    (Some("GET"), Some(domain)) => Request::Get(domain.to_string()),
                    (Some("DATA"), Some(offset)) => Request::Data(offset.parse().unwrap()),
                    _ => Request::Other(req.to_string()),
                }
            }
        }
    }
}
