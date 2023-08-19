# Alien

A simple operating system implemented in rust. The purpose is to explore how to use modules to build a complete os, so the system is composed of a series of independent modules. At present, the system already supports user-mode programs and some simple functions.

![image-20230815132104606](assert/image-20230815132104606.png)

## Project Structure

```
├── LICENSE
├── Makefile                (编译命令)
├── README.md               (readme)
├── apps                    (rust程序)
├── assert
├── boot                    (内核启动代码)
├── doc                     (开发文档与内核相关模块文档)
├── kernel					
	├── Cargo.toml
    ├── build.rs            (系统调用+调式符号生成脚本)
    └── src			
        ├── arch            (riscv相关代码)
        ├── board           (板级设备扫描注册)
        ├── config.rs       (内核配置)
        ├── device          (设备注册管理)
        ├── driver          (设备驱动)
        ├── error.rs        (内核错误码定义)
        ├── fs              (文件系统相关)
        ├── gui.rs          (gui显示相关)
        ├── interrupt       (外中断相关)
        ├── ipc             (进程间通信模块)
        ├── lib.rs          (内核代码模块导出)
        ├── memory          (内存管理)
        ├── net             (网络模块)
        ├── panic.rs        (堆栈回溯)
        ├── print           (内核输入输出)
        ├── sbi.rs          (SBI系统调用)
        ├── sync            (同步原语)
        ├── sys.rs          (内核运行信息)
        ├── syscall.rs      (系统调用表)
        ├── system.rs       (机器信息)
        ├── task            (进程/线程管理)
        ├── timer           (计时器)
        ├── trace           (堆栈回溯)
        └── trap            (异常处理)
├── modules                 (独立模块)
├── rust-toolchain.toml		
├── sdcard                  (测试程序)
├── test                    (初赛测试)
├── tools                   (一些dts文件)
└── userlibc                (rust lib库)
```



## Run

1. install qemu 7.0.0
2. install rust nightly

```
make run LOG= SMP=1
```

如果只想重新构建`kernel`而不改变测试程序，可以使用：

```
make build LOG= SMP=1
```

使用已经编译好的程序运行而不是再次编译可以使用：

```
make fake_run SMP=1
```

可以指定LOG=ERROR/WARN/INFO/DEBUG/TRACE开启调式

### Run with Gui

```
make run LOG=WARN SMP=1 GUI=y
slint or guitest or todo or printdemo or memorygame or ...
```

### Run VisionFive2

```
make sdcard
// 制作fat32
make vf2 LOG=WARN VF2=y SMP=2
// 生成testos.bin
// 这里smp=2 表示的是单核启动，对于u74-mc处理器，0号核不会被启动，从1号开始。
```

### Run cv1811h

```
make sdcard 
// 制作fat32
make vf2 LOG=WARN CV1811h=y SMP=1 
// 等待修复
```

## Run Unmatched

```
make sdcard
// 制作fat32
make unmatched LOG= UNMATCHED=y SMP=2
// 生成testos.bin
// 这里smp=2 表示的是单核启动，对于u74-mc处理器，0号核不会被启动，从1号开始。
```

目前cv1811h开发板可以上板启动，但是我们暂时没有处理其需要的特殊页表项。对于visionfive2和unmatched，可以启动并运行bash。

## GDB

1. `gdb-server`
2. `gdb-client`

## Doc

- [决赛规划](./doc/target.md)
- [开发日志](./doc/开发日志.md)
- [系统架构](./doc/系统架构.md)
- [设备管理](./doc/设备管理.md)
- 设备驱动
  - [串口设备](./doc/uart.md)
  - [SDIO](./doc/sdio.md)
  - [PLIC](./doc/plic.md)
- 文件系统
  - [文件系统接口](./doc/fs.md)
- [测试支持情况](./doc/test.md)
- [网络支持](./doc/net.md)
- 应用程序
  - [bash](./doc/bash.md)
  - [lmbench](./doc/lmbench.md)
  - [redis](./doc/redis.md)
  - [unixbench](./doc/unixbench.md)

- 杂项
  - [开发板相关](./doc/boot.md)
  - [fat32bug](./doc/fat32.md)
  - [内存管理](./doc/memory.md)
  - [slab](https://github.com/os-module/rslab/tree/main)
  - [dbfs](https://github.com/Godones/dbfs2)
  - [物理页帧分配器](./modules/pager/README.md)
- 更多: 查看[系统架构](./doc/系统架构.md)中module部分各个子模块的详细说明



## Feature

- [x] Thread/Mutil-core
- [x] full vfs
- [x] fat32
- [x] dbfs
- [x] Mutex
- [x] sleep task queue
- [x] uart task queue
- [x] a simple shell
- [x] memory management
- [x] process management
- [x] stack trace
- [x] signal
- [ ] ....



## App/Test

- [x] libc-test
- [x] busybox
- [x] lua
- [x] lmbench
- [x] iozone
- [x] cyclictest
- [x] libc-bench
- [x] unixbench
- [x] netperf
- [x] iperf
- [x] bash
- [x] redis
- [x] sqlite3
- [x] slint gui
- [x] embedded graphic gui



## Reference

- rCoreTutorial-v3 http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter0/index.html
- Maturin https://gitlab.eduxiji.net/scPointer/maturin
- Redox https://gitlab.redox-os.org/redox-os/
- [Files · master · FTL OS / OSKernel2022-FTLOS · GitLab (eduxiji.net)](https://gitlab.eduxiji.net/DarkAngelEX/oskernel2022-ftlos/-/tree/master)

