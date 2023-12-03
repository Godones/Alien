# 改进的阶段成果

自从上次发布第一篇[blog](./new-vfs.md)以来，我们对内核的部分继续做了很多改进。这主要包含了几部分的内容，首先是对文件系统的改进已经完成，其次是对网络相关的部分的进一步模块化，在用户态，我们新增了一个模块以方便使用`slint` gui框架开发。

## 文件系统的改进

目前内核将会使用位于[rvfs](https://github.com/os-module/rvfs)中的几个文件系统来构建系统中的目录结构

```rust
vfscore = { git = "https://github.com/os-module/rvfs.git", features = [
    "linux_error",
] }
devfs = { git = "https://github.com/os-module/rvfs.git" }
dynfs = { git = "https://github.com/os-module/rvfs.git" }
ramfs = { git = "https://github.com/os-module/rvfs.git" }
fat-vfs = { git = "https://github.com/os-module/rvfs.git" }
```

目前系统的目录结构大致如下:

```
|
|--root
|--var
	|--log
	|--tmp
	|--run
|--etc
|--dev
|--proc
|--sys
|--bin
|--tmp
```

在每个目录下，还有一些文件，这里的许多文件在系统启动时被创建，位于bin目录下的则是一些测试以及几个linux应用程序。在内部，我们对大多数文件系统相关的系统调用进行了重新实现，并统一了错误处理，大多数的文件系统调用如下面的形式：

```rust
#[syscall_func(79)]
pub fn sys_fstateat(
    dir_fd: isize,
    path: *const u8,
    stat: *mut u8,
    flag: usize,
) -> AlienResult<isize>
```

在`vfscore`内部，有自定义的`VfsError`，因为我们引入`linux_error` feature，因此`VfsError`实现了到`LinuxErrno` 的转换，使用`Into<T>` trait。在系统中，`AlienResult<T>` 被直接定义为`Result<T, AlienError>`, 这大大方便了代码的编写，我们不再需要进行`unwrap`或者额外的判断，一个`?`可以向上传递大多数的错误。

在注册所有设备之后，我们才会建立文件系统的目录结构，在这个过程中，所有的设备会形成一个设备文件，生成到`/dev/`目录下。这些设备实现了`VfsInode` 接口，因此对设备的访问现在与普通文件变得一致。

对内核文件系统相关系统调用的实现同时也促进我们对`rvfs`模块的改进，我们增加了一些有用的函数，并检查出一些bug，对于错误的处理也变得更加合理，这对后续新的文件系统支持大有益处。

## 网络模块

与文件系统类似，网络子系统的协议栈部分也被我们移出了内核代码中，内核中只保留网络系统调用实现。协议栈主要分为两部分，下层是网卡的抽象，上层是基于[smoltcp](https://github.com/smoltcp-rs/smoltcp)的实现。大部分的代码来自[Arceos](https://github.com/rcore-os/arceos) 和[smoltcp](https://github.com/smoltcp-rs/smoltcp)，并做了一些修改，这个部分作为一个独立的仓库存在[simple-net](https://github.com/os-module/simple-net)。

对于网卡设备的抽象，我们定义了一个接口:

```rust
pub trait NetBufOps: Any {
    fn packet(&self) -> &[u8];
    fn packet_mut(&mut self) -> &mut [u8];
    fn packet_len(&self) -> usize;
}

/// Operations that require a network device (NIC) driver to implement.
pub trait NetDriverOps: Send + Sync {
    fn medium(&self) -> Medium;
    /// The ethernet address of the NIC.
    fn mac_address(&self) -> EthernetAddress;

    /// Whether can transmit packets.
    fn can_transmit(&self) -> bool;

    /// Whether can receive packets.
    fn can_receive(&self) -> bool;

    /// Size of the receive queue.
    fn rx_queue_size(&self) -> usize;

    /// Size of the transmit queue.
    fn tx_queue_size(&self) -> usize;

    /// Gives back the `rx_buf` to the receive queue for later receiving.
    ///
    /// `rx_buf` should be the same as the one returned by
    /// [`NetDriverOps::receive`].
    fn recycle_rx_buffer(&mut self, rx_buf: Box<dyn NetBufOps>) -> Result<(), NetError>;

    /// Poll the transmit queue and gives back the buffers for previous transmiting.
    /// returns [`DevResult`].
    fn recycle_tx_buffers(&mut self) -> Result<(), NetError>;

    /// Transmits a packet in the buffer to the network, without blocking,
    /// returns [`DevResult`].
    fn transmit(&mut self, tx_buf: Box<dyn NetBufOps>) -> Result<(), NetError>;

    /// Receives a packet from the network and store it in the [`NetBuf`],
    /// returns the buffer.
    ///
    /// Before receiving, the driver should have already populated some buffers
    /// in the receive queue by [`NetDriverOps::recycle_rx_buffer`].
    ///
    /// If currently no incomming packets, returns an error with type
    /// [`DevError::Again`].
    fn receive(&mut self) -> Result<Box<dyn NetBufOps>, NetError>;

    /// Allocate a memory buffer of a specified size for network transmission,
    /// returns [`DevResult`]
    fn alloc_tx_buffer(&mut self, size: usize) -> Result<Box<dyn NetBufOps>, NetError>;
}
```

这个接口定义了网卡需要支持的功能，我们在此基础之上实现了`loopback`和`virtio-net`两个设备，`loopback`设备所作的工作非常简单，只是把发送的包原封不动地放到队列中。`virtio-net`设备是对使用`virtio`协议的网卡设备的封装，其使用了[virtio-drivers ](https://github.com/rcore-os/virtio-drivers) 来完成更多的工作。

在[netcore](https://github.com/os-module/simple-net)中，我们为`smoltcp` 实现一些定义的接口，从而可以驱动`smoltcp`协议栈工作。在此基础上，实现了`tcp`与`udp`两个数据结构。

虽然协议栈目前被移出了内核之外，但是模块化程度依然不够，而且不够灵活。目前`netcore`的入口函数形式如下:

```rust
pub fn init_net(
    device: Box<dyn NetDriverOps>,
    kernel_func: Arc<dyn KernelNetFunc>,
    ip: IpAddress,
    gate_way: IpAddress,
    test: bool,
)
```

入口函数会初始化一些必要的数据结构，这些数据结构目前以全局变量的形式存在：

```rust
pub static NET_INTERFACE: Once<NetInterfaceWrapper> = Once::new();
pub static SOCKET_SET: Lazy<SocketSetWrapper> = Lazy::new(SocketSetWrapper::new);
pub static LISTENING_TABLE: Lazy<ListenTable> = Lazy::new(ListenTable::new);
```

这导致了我们只能使用一个网卡设备，但对于准备的大多数测试，我们只能在回环设备下完成。这导致了qemu的`virtio`设备无法被有效利用。

初次之外，对于互斥锁的使用，我们直接引用了内核中使用的`kernel-sync`:

```
[dependencies]
kernel-sync = { git = "https://github.com/os-module/kernel-sync.git" }
```

这显然限制了其灵活性，因此在后面的改进当中，我们需要进一步的去除全局变量的使用，支持多个网卡设备，同时将互斥锁的使用更换为`lock-api`。

## virt2slint

我们在内核中添加了一些GUI相关的支持。要在内核中添加GUI设备，我们需要添加一些qemu参数:

```
QEMU_ARGS += -device virtio-gpu-device \
			 -device virtio-tablet-device \
			 -device virtio-keyboard-device
```

在内核中，我们使用已有的`virtio`驱动程序添加了对这些设备的使用。并提供了两个简单的系统调用来向用户态提供显示缓冲区和输入事件：

```rust
#[syscall_func(2000)]
pub fn sys_framebuffer() -> isize
#[syscall_func(2001)]
pub fn sys_framebuffer_flush() -> isize
#[syscall_func(2002)]
pub fn sys_event_get(event_buf: *mut u64, len: usize) -> isize 
```

在用户态，为了简化开发难度，我们使用了[slint](https://slint.dev/)框架来提供gui的开发，对于嵌入式系统，这需要提供一些底层接口的实现，比如事件的传递，缓冲区的绘制。

`virt2slint` 这个crate提供了`virtio`的输入事件到`slint`的输入事件的转换支持，有了这个支持，就可以方便地驱动起一些嵌入式系统的GUI了。

在apps目录下，我们移植了`slint`官方提供的几个嵌入式应用程序:

```
printdemo
memory-game
slint
sysinfo
todo
```

## 杂项

除了上面这些较大的改动之外，内核中还有一些其它的改进，比如一些无用代码的删除以及一些重构。

## 下一步

目前我们已经完成了内核中已有模块的大多数重构，但内核中还有一些很多小的模块，这些模块因为其耦合性导致修改起来更加复杂，因此可能需要从其周围的依赖进行逐步的修改和删减。其中很重要的部分是页表的管理，这部分涉及到了写时复制、懒分配等比较复杂的策略。

