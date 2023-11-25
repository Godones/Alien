use netcore::{KernelNetFunc, NetInstant};

use crate::interrupt::DeviceBase;

use crate::timer::TimeSpec;

pub trait NetDevice: DeviceBase {}

#[cfg(feature = "net_test")]
pub mod nettest {

    use alloc::vec::Vec;
    use core::net::{IpAddr, SocketAddr};

    use netcore::tcp::TcpSocket;

    use crate::error::{AlienError, AlienResult};

    /// A TCP stream between a local and a remote socket.
    pub struct TcpStream(TcpSocket);

    /// A TCP socket server, listening for connections.
    pub struct TcpListener(TcpSocket);

    impl TcpStream {
        pub fn read(&mut self, buf: &mut [u8]) -> AlienResult<usize> {
            self.0.recv(buf).map_err(|_| AlienError::Other)
        }
        pub fn write_all(&mut self, buf: &[u8]) -> AlienResult<()> {
            self.0.send(buf).map_err(|_| AlienError::Other).map(|_| ())
        }
    }

    impl TcpListener {
        pub fn bind(addr: SocketAddr) -> AlienResult<TcpListener> {
            let socket = TcpSocket::new();
            socket.bind(addr).map_err(|_| AlienError::Other)?;
            socket.listen().map_err(|_| AlienError::Other)?;
            Ok(TcpListener(socket))
        }
        pub fn local_addr(&self) -> AlienResult<SocketAddr> {
            self.0.local_addr().map_err(|_| AlienError::Other)
        }

        pub fn accept(&self) -> AlienResult<(TcpStream, SocketAddr)> {
            let socket = self.0.accept().map_err(|_| AlienError::Other)?;
            let addr = socket.peer_addr().map_err(|_| AlienError::Other)?;
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
            let n = stream.read(&mut buf).map_err(|_| AlienError::Other)?;
            if n == 0 {
                return Ok(());
            }
            stream
                .write_all(reverse(&buf[..n]).as_slice())
                .map_err(|_| AlienError::Other)?;
        }
    }

    pub fn accept_loop() {
        let local_addr = SocketAddr::new(
            IpAddr::parse_ascii(LOCAL_IP.as_bytes()).unwrap(),
            LOCAL_PORT,
        );
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

#[derive(Debug, Default)]
pub struct NetNeedFunc;

impl KernelNetFunc for NetNeedFunc {
    fn now(&self) -> NetInstant {
        let time_spec = TimeSpec::now();
        NetInstant {
            micros: time_spec.tv_sec as i64 * 1000_000 + time_spec.tv_nsec as i64 / 1000,
        }
    }
    #[cfg(feature = "net_test")]
    fn yield_now(&self) {
        // do_suspend();
    }
    #[cfg(not(feature = "net_test"))]
    fn yield_now(&self) -> bool {
        use crate::task::current_task;
        use crate::task::do_suspend;
        do_suspend();
        // interrupt by signal
        let task = current_task().unwrap();
        let task_inner = task.access_inner();
        let receiver = task_inner.signal_receivers.lock();
        if receiver.have_signal() {
            return true;
        }
        false
    }
}
