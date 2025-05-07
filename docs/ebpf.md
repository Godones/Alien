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







