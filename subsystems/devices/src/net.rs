#[cfg(feature = "net_test")]
pub mod nettest {
    use crate::net::port::neterror2alien;
    use alloc::vec::Vec;
    use constants::AlienResult;
    use core::net::{IpAddr, SocketAddr};
    use netcore::tcp::TcpSocket;

    /// A TCP stream between a local and a remote socket.
    pub struct TcpStream(TcpSocket);

    /// A TCP socket server, listening for connections.
    pub struct TcpListener(TcpSocket);

    impl TcpStream {
        pub fn read(&mut self, buf: &mut [u8]) -> AlienResult<usize> {
            self.0.recv(buf).map_err(neterror2alien)
        }
        pub fn write_all(&mut self, buf: &[u8]) -> AlienResult<usize> {
            self.0.send(buf).map_err(neterror2alien)
        }
    }

    impl TcpListener {
        pub fn bind(addr: SocketAddr) -> AlienResult<TcpListener> {
            let socket = TcpSocket::new();
            socket.bind(addr).map_err(neterror2alien)?;
            socket.listen().map_err(neterror2alien)?;
            Ok(TcpListener(socket))
        }
        pub fn local_addr(&self) -> AlienResult<SocketAddr> {
            self.0.local_addr().map_err(neterror2alien)
        }

        pub fn accept(&self) -> AlienResult<(TcpStream, SocketAddr)> {
            let socket = self.0.accept().map_err(neterror2alien)?;
            let addr = socket.peer_addr().map_err(neterror2alien)?;
            Ok((TcpStream(socket), addr))
        }
    }

    const LOCAL_IP: &str = "0.0.0.0";
    const LOCAL_PORT: u16 = 5555;

    fn reverse(buf: &[u8]) -> Vec<u8> {
        let mut lines = buf
            .split(|&b| b == b'\n')
            .map(Vec::from)
            .collect::<Vec<_>>();
        for line in lines.iter_mut() {
            line.reverse();
        }
        lines.join(&b'\n')
    }

    fn echo_server(mut stream: TcpStream) -> AlienResult<()> {
        let mut buf = [0u8; 1024];
        loop {
            let n = stream.read(&mut buf).unwrap();
            if n == 0 {
                return Ok(());
            }
            stream.write_all(reverse(&buf[..n]).as_slice()).unwrap();
        }
    }

    pub fn accept_loop() {
        let local_addr = SocketAddr::new(IpAddr::V4(LOCAL_IP.parse().unwrap()), LOCAL_PORT);
        let listener = TcpListener::bind(local_addr).unwrap();
        println!("listen on: {}", listener.local_addr().unwrap());
        let mut i = 0;
        loop {
            match listener.accept() {
                Ok((stream, addr)) => {
                    println!("new client {}: {}", i, addr);
                    match echo_server(stream) {
                        Err(e) => {
                            println!("client connection error: {:?}", e);
                        }
                        Ok(()) => {
                            println!("client {} closed successfully", i);
                        }
                    }
                }
                Err(e) => panic!("couldn't get client: {:?}", e),
            }
            i += 1;
        }
    }
}
