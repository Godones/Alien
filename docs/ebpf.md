# eBPF Support

这篇博客旨在记录如何在宏内核os中添加eBPF的支持，以及使用 [Aya](https://github.com/aya-rs/aya) 库编写一个运行在kprobes上的eBPF程序。

## Steps to Enable eBPF

1. Support kprobes
2. Allow to run tokio runtime at user space
3. Add basic syscalls and structs for eBPF
4. Add Aya app



## 1.Support kprobes

为了让eBPF程序附着在内核的hook点上，我们需要为内核添加一些跟踪点，其中kprobe是比较简单的一类，因此我们以kprobe来作为基本支持的演示。

[kprobe](https://github.com/Godones/ext_ebpf) 库中提供了kprobe的基本实现，允许我们在特定的地址上进行探测。同时，该库提供了一个重要的`ProbeArgs`  trait 对内核的Trapframe进行了抽象，并规范了需要实现的功能。

为了在内核中引入这个库，需要完成以下几步：

1. 若内核中没有处理break异常，需要添加相应的处理

   1. 如果在x86架构下，还需要添加单步异常的处理

   > [!NOTE]
   >
   > x86架构下使用break异常和单步异常实现kprobe，而其它架构下没有单步异常，因此是使用两个break异常来实现kprobe

2. 为内核的Trapframe实现`ProbeArgs`

3. 使用kprobe库提供的`KprobeManager` 和 `KprobePointList` 定义kprobe相关的全局数据

4. 在异常处理中调用kprobe库提供的`kprobe_handler_from_break` 或者 `kprobe_handler_from_debug`

   > [!NOTE]
   >
   > - 在x86架构下，break异常调用`kprobe_handler_from_break`, 单步异常调用`kprobe_handler_from_debug`
   > - 在其它架构下(riscv64/loongarch64)，只有break异常，只需要调用`kprobe_handler_from_break`，krobe库内部会自行调用`kprobe_handler_from_debug`

   1. 需要注意，对于有kprobe挂载的break点，kprobe库会更新Trapframe，因此内核的异常处理无须再更新pc的值

5. 需要允许kernel修改代码段，并允许在数据段中执行代码

   1. kprobe库定义了一个`KprobeAuxiliaryOps` 来向内核请求设置代码段可写并分配可执行的内存，因此需要根据内核的设计实现这个trait，比如修改内核代码段中的页表标志或者分配一段可执行内存



预估新增代码量: 

1. 10-20(Trapframe实现`ProbeArgs`)
2. 10-15(新增break的处理)
3. 设置可写代码段以及分配可执行内存(20~) 在某些系统上，内核的代码段和内存都具有读写可执行属性，这里的实现就可以为空。



## 2.Support eBPF

### Tokio 或其它异步运行时

为了后面支持使用已有的eBPF框架来编写程序，我们可能需要首先支持一些功能，比如为了支持Rust社区的Aya框架，我们首先需要允许运行异步程序。Rust社区比较出名的两个异步运行时是Tokio 和 smol

当然，为了简单起见，不需要完整支持异步运行时，在Alien中，目前只支持**单线程运行的异步运行时**。

可能需要的系统调用：

- eventfd
- epoll系列

### 系统调用支持

为了支持运行eBPF程序，最重要的系统调用是[bpf()](https://man7.org/linux/man-pages/man2/bpf.2.html) ，该系统调用下包含大量的子命令，但eBPF的初步支持并不需要所有命令都实现，需要实现的基础命令如下：

```
bpf_cmd::BPF_MAP_CREATE
bpf_cmd::BPF_MAP_UPDATE_ELEM
bpf_cmd::BPF_MAP_LOOKUP_ELEM
bpf_cmd::BPF_MAP_GET_NEXT_KEY
pf_cmd::BPF_MAP_DELETE_ELEM
bpf_cmd::BPF_MAP_LOOKUP_AND_DELETE_ELEM
bpf_cmd::BPF_MAP_LOOKUP_BATCH
bpf_cmd::BPF_MAP_FREEZE
bpf_cmd::BPF_PROG_LOAD
```

其它的命令可以暂时不用实现，需要注意的是，这里的大多数功能在[bpf-basic](https://github.com/Godones/ext_ebpf) 库中已经实现。bpf-basic库定义了几个内核需要实现的trait，只需要实现这些trait就可以调用这些功能。

- PerCpuVariantsOps 和 PerCpuVariants两个trait定义了内核创建per-cpu变量的方式，这对per-cpu类型的map非常重要
- KernelAuxiliaryOps则定义了更多功能，比如内核需要根据fd返回bpf-basic中定义的UnifiedMap，以及一些buf转换函数

另一个很重要的系统调用是[perf_event_open](https://man7.org/linux/man-pages/man2/perf_event_open.2.html)，该系统调用跟eBPF相关的部分是两个子命令：

```
perf_type_id::PERF_TYPE_MAX
perf_type_id::PERF_TYPE_SOFTWARE
```

- PERF_TYPE_MAX 用来创建一个kprobe，并在稍后将eBPF程序附着到该hook点上
- PERF_TYPE_SOFTWARE 用来创建一个环形缓冲区，eBPF程序的输出将会输出到该缓冲区中

其中bpf-basic库提供了环形缓冲区的实现

内核需要提供KprobePerfEvent的抽象，这主要包含将kprobe文件与ebpf程序文件关联起来，并且将eBPF的执行作为回调函数注册到kprobe上。kprobe库中定义了CallBackFunc trait。



### 文件系统支持

文件系统的支持分为两部分：

1. 内核中，不管是eBPF程序，还是eBPF程序中使用的Map，内核都将其作为一个文件对待，因此，内核需要为这些数据结构实现内核定义的File抽象接口
2. 用户态的Aya库会读取一些特殊的文件，并根据这些结果判断使用什么方法创建kprobe，其它的文件则用于计算cpu的数量

详细来说，内核需要将eBPF相关的数据结果封装，并实现相关的File接口：

- BpfMap，这是对bpf-basic库中的UnifiedMap的封装
- BpfProg，这是对bpf-basic库中的BpfProgMeta和EBPFPreProcessor的封装，BpfProgMeta记录了eBPF程序的基本信息，EBPFPreProcessor则是对eBPF程序进行预处理后的结果，这些预处理包括对eBPF程序的特定指令进行重定位操作，从而将Map的FD转换为对应的指针
- PerfEvent，其内部实现了两种类型的perf event，一个是KprobePerfEvent，一个是BpfPerfEvent。PerfEvent需要关注的是ioctl、poll、mmap的实现
  - ioctl用于使能/关闭kprobe，以及在kprobe上挂载eBPF程序
  - poll用于检查环形缓冲区是否可读
  - mmap用于映射给环形缓冲区建立映射

对于特殊文件，内核需要在在/sys目录下创建特定的文件：

- /sys/bus/event_source/devices/kprobe/type 用于识别kprobe的 type
- /sys/devices/system/cpu/online  当前的在线cpu数量
- /sys/devices/system/cpu/possible 系统总的cpu数量

用户态的Aya库会检查当前的内核版本，这可以通过两种方式，一种是读取特定文件，这在ubuntu系统上使用，一种是使用uname返回的结果，为了简单，我们可以修改uname返回的结果。

内核需要设定uname返回结果中内核版本>=5.4.0，这样Aya才会使用perf系统调用创建kprobe



### 内核符号表支持

由于用户态在创建kprobe时是根据内核符号名称进行创建的，这需要内核拥有根据符号查找对应地址的功能。为此，需要将内核符号信息提取出来并供内核使用。

[ksym](https://github.com/Godones/ext_ebpf)模块了帮助内核生成符号信息和搜索符号信息的能力。为了使用该模块，我们将内核的编译分为两个阶段：

1. 首次编译内核，在使用链接器链接后，使用ksym工具生成符号表信息，以汇编格式
2. 将汇编符号表信息编译
3. 第二次链接， 将该符号表信息和内核一起链接





