# NetWork

## linux的socket编程



## 与socket相关的系统调用

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
// 创建一对未绑定的socket套接字，该对套接字可以用于全双工通信，或者用于父子进程之间的通信）
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
// address: 指明存储的有关绑定信息（sockaddr结构）的地址（sockaddr结构包括地址组信息address_family和要绑定的地址信息socket_address）
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
fn sys_accept(socket: usize, address: *const usize, address_len: *usize) -> SysResult;



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
// 发送消息。当面向连接时，dest_addr被忽略；当非面向连接时，消息发送给dest_addr。
//
// socket: 指明要操作socket的文件描述符id
// buffer: 指明接收message的缓冲区的首地址
// length: 指明能接收message的最大长度
// flags: 指明接收message的类型
// address: 指明信息发送方的相关信息（sockaddr结构）的保存地址
// address_len: 指明address的（sockaddr）结构体的长度的地址
// 返回: 如果发送成功，返回接收message的字节数；否则返回错误信息
pub fn sys_recvfrom(socket: usize, buffer: *mut u8, length: usize, flags:i32, src_addr: *mut usize, address_len: *mut u32) -> isize;


// sys_recv:
// 同sys_recvfrom, 简化版
fn sys_recv(socket: usize, buffer: *const usize, length: usize, flags:usize) -> SysResult;


// sys_setsockopt: syscall_id 208
// sys_getsockopt: syscall_id 209
// sys_sendmsg:  syscall_id 211
// sys_recvmsg:  syscall_id 212



```

#### 一些必要的结构

```rust

struct sockaddr_in {
    short int sin_family;         // AF_INET
    unsigned short int sin_port;  // 端口号
    struct in_addr  sin_addr;     // IP地址
};

```





## 参考资料

[socket.h](https://pubs.opengroup.org/onlinepubs/7908799/xns/syssocket.h.html)

[Matruin实现 socket](https://gitlab.eduxiji.net/scPointer/maturin/-/tree/master/kernel/src/file/socket)

[rCore-N net](http0s://github.com/CtrlZ233/rCore-N/tree/main/os/src/net)