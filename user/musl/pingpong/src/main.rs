use std::net::UdpSocket;

fn main() {
    let client = UdpSocket::bind("127.0.0.1:2000").expect("bind failed");
    let server = UdpSocket::bind("127.0.0.1:2001").expect("bind failed");
    let client_thread = std::thread::spawn(move || {
        let mut buf = [0; 10];
        let mut count = 0;
        for _ in 0..5 {
            let msg = format!("ping {}", count);
            client
                .send_to(msg.as_bytes(), "127.0.0.1:2001")
                .expect("send_to failed");
            client.recv_from(&mut buf).expect("recv_from failed");
            println!("client received: {}", String::from_utf8_lossy(&buf));
            count += 1;
        }
        println!("client finished");
    });
    let server_thread = std::thread::spawn(move || {
        let mut buf = [0; 10];
        let mut count = 0;
        for _ in 0..5 {
            server.recv_from(&mut buf).expect("recv_from failed");
            println!("server received: {}", String::from_utf8_lossy(&buf));
            let msg = format!("pong {}", count);
            server
                .send_to(msg.as_bytes(), "127.0.0.1:2000")
                .expect("send_to failed");
            count += 1;
        }
        println!("server finished");
    });
    client_thread.join().expect("join failed");
    server_thread.join().expect("join failed");
}
