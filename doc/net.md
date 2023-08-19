# NetWork

### Todo List

+ [x] 封装Interface的poll方法，使其易于调用
    + [x] 完善DeviceWrapper和InterfaceWrapper结构
+ [x] 实现各系统调用
    + [x] socket::new
    + [x] udp: bind + send + recieve
    + [x] ListenTable 结构
    + [x] tcp
        + [x] sever:  bind + listen + accept + recv + send
        + [x] client:  connect + recv + send
    + [x] close
    + [x] shutdown
+ [x] 重构目前的IpAddr，使用smoltcp中已有的结构

+ [ ] 支持DNS



## 相关设计

### 启动网络功能

在启动时附带上`NET=y`(例如`make run LOG=WARN img=fat32 SMP=1 GUI=n NET=y`)即可启动网络功能

### 启动虚拟网卡
当启动时附带上`NET=y`时，会在Qemu的参数中加入`netdev user,id=net -device virtio-net-device,netdev=net`启动虚拟网卡
```makefile
ifeq ($(NET),y)
QEMU_ARGS += -netdev user,id=net \
			 -device virtio-net-device,netdev=net
endif
```

### 初始化网卡设备和接口
```rust
/// driver/dtb.rs
fn virtio_device(transport: MmioTransport, addr: usize, irq: usize) {
    match transport.device_type() {
        // ...
        DeviceType::Network => virto_net(transport),
        // ...
    }
}

fn virto_net(transport: MmioTransport){
    let net = VirtIONet::<HalImpl, MmioTransport, NET_QUEUE_SIZE>::new(transport, NET_BUFFER_LEN)
        .expect("failed to create net driver");
    println!("MAC address: {:02x?}", net.mac_address());
    NET_DEVICE.call_once(|| VirtIONetWrapper::new(net));
    println!("virtio-net init finished");
    /// ...
    
}

```

使用`virtio`中的`VirtIONet`初始化网卡设备`NET_DEVICE`。此时需要将`VirtIONet`包装成`VirtIONetWrapper`，一方面为了满足多线程共享，另一方面要为`VirtIONetWrapper`实现`smoltcp`中的`Device`特征，以便使用`VirtIONetWrapper`初始化`smoltcp`上一层的`Interface`(`Interface`结构体是网络接口的抽象表示，它提供了与该接口相关的功能和操作)。其中对于`Interface`结构主要使用的方法为poll()，它的作用包括：
+ 检查接口是否有待发送的数据包，如果有，则将这些数据包发送到网络。
+ 检查接口是否接收到新的数据包，如果有，则将这些数据包传递给网络栈进行处理。
+ 处理接口的定时器事件，例如重新发送未确认的数据包、更新路由表等。
+ 处理接口的其他网络事件，例如连接建立、连接关闭等。

在socket每次需要发送或接收数据包前，都需要调用poll()，之后将对其进行封装，方便socket调用。



### smoltcp中的相关socket结构

`smoltcp`是一个用于嵌入式系统的轻量级TCP/IP协议栈。它提供了一组简单易用的API来实现网络通信功能。

#### SocketSet

 用于管理套接字（socket）的集合，提供方便的方式来创建、配置和管理多个套接字。通过`Interface`结构，可以配置和管理网络接口的地址、路由表，发送和接收IP数据包，以及处理邻居设备的信息。它是`smoltcp`库中用于网络通信的重要组件之一。

`SocketSet`结构的一些重要字段和方法：

- `sockets: &'a mut [Option<Socket<'b>>]`：表示套接字的集合，以`Option<Socket>`的形式存储在数组中。
- `timestamp: Instant`：表示当前时间戳，用于套接字的超时处理。
- `storage: &'b mut [u8]`：表示套接字的存储空间，用于存储套接字的状态和数据。

`SocketSet`结构提供了一些方法来管理套接字集合，包括：

- `new(sockets: &'a mut [Option<Socket<'b>>], storage: &'b mut [u8]) -> SocketSet<'a, 'b>`：创建一个新的`SocketSet`对象，指定套接字集合和存储空间。
- `get::<T>(&mut self, handle: SocketHandle) -> Option<&mut T>`：根据套接字句柄获取指定类型的套接字的可变引用。
- `get::<T>(&self, handle: SocketHandle) -> Option<&T>`：根据套接字句柄获取指定类型的套接字的不可变引用。
- `add<S: SocketLike>(&mut self, socket: S) -> Result<SocketHandle, S>`：向套接字集合中添加一个新的套接字，并返回套接字句柄。
- `remove(&mut self, handle: SocketHandle) -> Option<Socket<'b>>`：从套接字集合中移除指定句柄的套接字，并返回移除的套接字。
- `poll(&mut self, device: &mut dyn Device, timestamp: Instant) -> Result<(), Error>`：轮询套接字集合，处理待发送和接收的数据，并更新套接字的状态。

后面将其封装成`SocketSetWrapper`，并提供快速创建TCP、UDP套接字并将其放入集合中的接口。



#### TcpSocket

用于表示TCP套接字。

`TcpSocket`结构体包含了以下字段：

- `state`：表示TCP套接字的状态，可以是`Closed`、`Listen`、`SynSent`、`SynReceived`、`Established`、`FinWait1`、`FinWait2`、`Closing`、`TimeWait`、`CloseWait`或`LastAck`。
- `local_endpoint`：表示本地端点，即本地IP地址和端口号。
- `remote_endpoint`：表示远程端点，即远程IP地址和端口号。
- `receive_queue`：表示接收数据的队列，存储接收到的数据段。
- `send_queue`：表示发送数据的队列，存储待发送的数据段。
- `send_unacked`：表示已发送但未收到确认的数据段。
- `send_next`：表示下一个要发送的数据段的序号。
- `send_wnd`：表示发送窗口大小，即还可以发送多少字节的数据。
- `receive_next`：表示下一个期望接收的数据段的序号。
- `receive_wnd`：表示接收窗口大小，即还可以接收多少字节的数据。

`TcpSocket`结构体还提供了一些方法，例如`connect`、`listen`、`accept`、`send`、`receive`等，用于实现TCP连接的建立、数据的发送和接收等操作。



#### UdpSocket

用于表示UDP套接字

下面是`UdpSocket`结构的一些重要字段和方法：

- `header: UdpHeader`：表示UDP头部的信息，包括源端口和目标端口。
- `endpoint: Option<SocketAddr>`：表示UDP套接字的端点，即源IP地址和源端口。
- `rx_buffer: [u8; UDP_RX_BUFFER_SIZE]`：UDP接收缓冲区，用于存储接收到的UDP数据。
- `tx_buffer: [u8; UDP_TX_BUFFER_SIZE]`：UDP发送缓冲区，用于存储待发送的UDP数据。

`UdpSocket`结构提供了一些方法来创建和管理UDP连接，包括：

- `new() -> UdpSocket<'a>`：创建一个新的`UdpSocket`对象。
- `bind(&mut self, endpoint: SocketAddr) -> Result<(), Error>`：将UDP套接字绑定到指定的端点，即指定源IP地址和源端口。
- `send(&mut self, dst: SocketAddr, payload: &[u8]) -> Result<(), Error>`：向指定的目标端点发送UDP数据。
- `receive(&mut self) -> Result<(SocketAddr, &[u8]), Error>`：接收UDP数据，并返回数据的源端点和有效负载。
- `can_send(&self) -> bool`：检查UDP套接字是否可以发送数据。
- `can_receive(&self) -> bool`：检查UDP套接字是否可以接收数据。

通过`UdpSocket`结构，可以创建一个UDP套接字，并使用`bind`方法将其绑定到指定的端点。然后，使用`send`方法向指定的目标端点发送UDP数据，使用`receive`方法接收UDP数据。还可以使用`can_send`和`can_receive`方法来检查套接字是否可以发送和接收数据。`UdpSocket`提供了一种简单而灵活的方式来创建和管理UDP连接，以进行无连接的、不可靠的数据传输。





### Alien中使用的SocketData结构

在Alien中，我们使用SocketData结构来记录Socket的相关信息，其定义如下：

```rust
pub struct SocketData {
    /// 用于在SocketSet中找到smoltcp下的socket套接字
    pub handler: Option<SocketHandle>,
    /// socket 通信域  
    pub domain: Domain,
    /// 连接类型
    pub s_type: SocketType,
    /// 具体的通信协议
    pub protocol: usize,
    /// 连接的远端服务器的信息
    pub peer_addr: IpAddr,
    /// 本地的信息
    pub local_addr: IpAddr,
    pub listening: bool,
    pub is_server: bool,
}
```

相关方法：

+ `is_tcp()`: 通过`s_type`属性判断该套接字是否为TCP Socket
+ `is_ucp()`: 通过`s_type`属性判断该套接字是否为UDP Socket

在相关调用中需要先判断套接字类型，然后对不同的类型采用不同的实现。





#### alien提供的系统调用

``` rust
pub fn sys_socket(domain: usize, socket_type: usize, protocol:usize) -> isize;

pub fn sys_sendto(
    socket: usize,
    message: *const u8,
    length: usize, 
    flags:i32, 
    dest_addr: *const usize, 
    dest_len: usize
    ) -> isize;

pub fn sys_recvfrom(
    socket: usize, 
    buffer: *mut u8, 
    length: usize, 
    flags:i32, 
    src_addr: *mut usize, 
    address_len: *mut u32
    ) -> isize;

pub fn sys_shutdown(socket: usize, how: usize) -> isize 

```



```rust
// sys_socket:   syscall_id 198
// 创建一个未绑定的socket套接字
// 
// domain: 指明套接字被创建的协议簇（包括文件路径协议簇和网络地址协议簇）
// type: 指明被创建的socket的类型
// protocol: 指明该socket应用于某一个特定的协议上。当确定了套接字使用的协议簇和类型，该参数可以取为0
// 返回: 如果创建成功则返回一个能在之后使用的文件描述符，否则返回错误信息。
pub fn sys_socket(domain: usize, socket_type: usize, protocol:usize) -> isize;



// sys_socket_pair:  syscall_id 199
// 创建一对未绑定的socket套接字，该对套接字可以用于全双工通信，或者用于父子进程之间的通信
// 如果向其中的一个socket写入后，再从该socket读时，就会发生阻塞。只能在另一个套接字中读。
// 往往和shutdown()配合使用
//
// domain: 指明套接字被创建的通信域（包括文件路径域和网络地址域）
// type: 指明被创建的socket的类型
// protocol: 指明该socket应用于某一个特定的协议上。取值为0时将导致此socket被视为未指明的默认类型，即实际需要的socket类型。
// sv[2]:  用于存放一对套接字的文件描述符。
// 返回: 如果创建成功则返回0，否则返回错误信息。
fn sys_socket_pair(domain: usize, socket_type: usize, protocol:usize, sv: *const usize ) -> SysResult;



// sys_bind: syscall_id 200
// 绑定socket的地址和端口
// 
// socket: 指明要操作socket的文件描述符id
// address: 指明存储有关绑定信息（sockaddr结构）的地址（sockaddr结构包括地址组信息address_family和要绑定的地址信息socket_address）
// address_len: address（即sockaddr结构）的长度。
// 返回: 执行成功则返回0，否则返回错误信息
fn sys_bind(socket: usize, address: *const usize, address_len: usize) -> SysResult;



// sys_listen: syscall_id 201
// 用于等待用户提交连接请求，一般用于bind之后，accept之前 
//
// socket: 指明要操作socket的文件描述符id
// backlog: 指明套接字侦听队列中正在处于半连接状态（等待accept）的请求数最大值。如果该值小于等于0，则自动调为0，同时也有最大值上限。
// 返回: 执行成功返回0，否则返回错误信息
fn sys_listen(socket: usize, backlog: usize) -> SysResult;



// sys_accept: syscall_id 202
// 用于取出套接字listen队列中的第一个连接，创建一个与指定套接字具有相同套接字类型的地址族的新套接字
// 新套接字用于传递数据，原套接字继续处理侦听队列中的连接请求。如果侦听队列中无请求，accept()将阻塞。
// 
// socket: 指明要操作socket的文件描述符id，需经过bind()和listen()处理
// address: 要么为空，要么指明连接的客户端相关信息（sockaddr结构）的保存地址
// address_len: 保存连接的客户端相关信息长度的地址。
// 返回: 执行成功则返回新的套接字的文件描述符，否则返回错误信息
fn sys_accept(socket: usize, address: *mut usize, address_len: *usize) -> SysResult;



// sys_connect: syscall_id 203
// 用于客户端请求在一个套接字上建立连接
// 
// socket: 指明要操作socket的文件描述符id
// address: 指明包含服务器地址和端口号的数据结构（sockaddr结构）的地址
// address_len: address（即sockaddr结构）的长度。
// 返回: 执行成功则返回0，否则返回错误信息
fn sys_connect(socket: usize, address: *const usize, address_len: usize) -> SysResult;



// sys_getsockname: syscall_id 204
// 查询一个套接字本地bind()的相关信息
//
// socket: 指明要操作socket的文件描述符id
// address: 指明相关信息（sockaddr结构）的保存地址
// address_len: 保存address长度的地址。
// 返回: 执行成功则返回0，否则返回错误信息
fn sys_getsockname(socket: usize, address: *const usize, address_len: *usize) -> SysResult;



// sys_getpeername: syscall_id 205
// 用于获取一个本地套接字所连接的远程服务器的信息。
// 
// socket: 指明要操作socket的文件描述符id
// address: 指明连接的客户端相关信息（sockaddr结构）的保存地址
// address_len: 保存address长度的地址。
// 返回: 执行成功则返回0，否则返回错误信息
fn sys_getpeername(socket: usize, address: *const usize, address_len: *usize) -> SysResult;




// sys_shutdown:  syscall_id 210 
// 关闭一个socket的发送操作或者接收操作
//
// socket: 指明要操作socket的文件描述符id
// how: 指明要关闭的操作：包括只关闭Read，只关闭Write，RW都关闭。
// 返回: 执行成功则返回0，否则返回错误信息
fn sys_shutdown(socket: usize, how: usize) -> SysResult;



// sys_sendto: syscall_id 206
// 发送消息。当面向连接时，dest_addr被忽略；当非面向连接时，消息发送给dest_addr。
//
// socket: 指明要操作socket的文件描述符id
// message: 指明要发送的message的首地址
// length: 指明message的长度
// flags: 指明message发送的类型
// dest_addr: 指明目的地的相关信息（sockaddr结构）的保存地址
// dest_len: 指明dest_addr的结构体的大小
// 返回: 如果发送成功，返回发送的字节数；否则返回错误信息
pub fn sys_sendto(socket: usize, message: *const u8, length: usize, flags:i32, dest_addr: *const usize, dest_len: usize) -> isize;

// sys_send:
// 同sys_sendto，简化版。
fn sys_send(socket: usize, message: *const usize, length: usize, flags:usize) -> SysResult;


// sys_recvfrom: syscall_id 207
// 接收消息。消息源地址的相关信息将会保存在src_addr所指向的位置处。
//
// socket: 指明要操作socket的文件描述符id
// buffer: 指明接收message的缓冲区的首地址
// length: 指明能接收message的最大长度
// flags: 指明接收message的类型
// src_addr: 指明信息发送方的相关信息（sockaddr结构）的保存地址
// address_len: 指明src_addr的（sockaddr）结构体的长度的地址
// 返回: 如果接收成功，返回接收message的字节数；否则返回错误信息
pub fn sys_recvfrom(socket: usize, buffer: *mut u8, length: usize, flags:i32, src_addr: *mut usize, address_len: *mut u32) -> isize;


// sys_recv:
// 同sys_recvfrom, 简化版
fn sys_recv(socket: usize, buffer: *const usize, length: usize, flags:usize) -> SysResult;


// sys_setsockopt: syscall_id 208
// sys_getsockopt: syscall_id 209
// sys_sendmsg:  syscall_id 211
// sys_recvmsg:  syscall_id 212



```





#### 参考资料

[socket.h](https://pubs.opengroup.org/onlinepubs/7908799/xns/syssocket.h.html)

[Matruin实现 socket](https://gitlab.eduxiji.net/scPointer/maturin/-/tree/master/kernel/src/file/socket)

[rCore-N net](https://github.com/CtrlZ233/rCore-N/tree/main/os/src/net)

[virtio-net](https://github.com/rcore-os/virtio-drivers/blob/master/src/device/net.rs)

[组件化OS--aceros的改进：支持和优化lwip网络协议栈](http://hub.fgit.ml/Centaurus99/arceos-lwip/blob/main/reports/final.md)

[详解：VirtIO Networking 虚拟网络设备实现架构](https://www.sdnlab.com/26199.html)

[[smoltcp - Rust - Docs.rs](https://docs.rs/smoltcp/)]