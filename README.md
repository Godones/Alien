# Alien

A simple operating system implemented in rust. The purpose is to explore how to use modules to build a complete os, so the system is composed of a series of independent modules. At present, the system already supports user-mode programs and some simple functions.

<img src="assert/image-20230815132104606.png" alt="image-20230815132104606" style="zoom:50%;" />

## Project Structure

```
├── LICENSE
├── Makefile                (编译命令)
├── README.md               (readme)
├── apps                    (rust程序)
├── assert
├── kernel                  (核心子系统)
├── doc                     (开发文档与内核相关模块文档)
├── subsystems							
    ├── arch            (riscv相关代码)
    ├── platform        (平台相关代码)
    ├── config		    (内核配置)
    ├── devices         (设备注册管理)
    ├── drivers         (设备驱动合集)
    ├── unwinder        (内核panic处理)
    ├── vfs             (虚拟文件系统)
    ├── interrupt       (外中断注册管理)
    ├── ipc             (进程间通信模块)
    ├── mem          	(内存管理)
    ├── knet            (网络模块)
    ├── ksync           (内核锁实现)
    ├── timer           (时间相关实现)
    ├── constants		(常量、错误定义)
    ├── device_interface(设备接口定义)
├── tests                   (测试程序)
├── tools                   (一些dts文件)
└── userlibc                (rust lib库)
```



## Run

1. install qemu 7.0.0
2. install rust nightly
3. install riscv64-linux-musl [toolchain](https://musl.cc/)

```
make help
```

```
# 一键运行
# 可以指定LOG=ERROR/WARN/INFO/DEBUG/TRACE开启调式
make run [LOG=] [SMP=]
```

构建测试程序镜像:

```
make sdcard [GUI=n/y]
```

如果只想重新构建`kernel`而不改变测试程序，可以使用：

```
make build LOG= [LOG=] [SMP=]
```

使用已经编译好的程序运行而不是再次编译可以使用：

```
# 确保和make build的SMP一致
make fake_run [SMP=]
```

运行测试(in bash)

```
> cd tests
> ls
> final_test
```

### Run with GUI (QEMU)

```
make run GUI=y
cd bin
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

## Run Unmatched

```
make sdcard
// 制作fat32
make unmatched LOG= UNMATCHED=y SMP=2
// 生成testos.bin
// 这里smp=2 表示的是单核启动，对于u74-mc处理器，0号核不会被启动，从1号开始。
```

## GDB

1. `make gdb-server`
2. `make gdb-client`

## [Doc](docs/doc/doc.md)



## Reference

- rCoreTutorial-v3 http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter0/index.html
- Maturin https://gitlab.eduxiji.net/scPointer/maturin
- Redox https://gitlab.redox-os.org/redox-os/
- [Files · master · FTL OS / OSKernel2022-FTLOS · GitLab (eduxiji.net)](https://gitlab.eduxiji.net/DarkAngelEX/oskernel2022-ftlos/-/tree/master)

