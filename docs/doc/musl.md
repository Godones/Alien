# 编译 riscv64gc-unknown-linux-musl rust程序

`riscv64gc-unknown-linux-musl` target 是Tier3，因此不能像`riscv64gc-unknown-none-elf` 一样直接使用。为了编译出静态链接的程序，需要修改相关的参数。

在user/musl/.cargo目录下，展示了必要的配置。

```
[target.riscv64gc-unknown-linux-musl]
rustflags = [
    "-L/opt/riscv-musl/lib/gcc/riscv64-linux-musl/11.2.1/",
    "-L/opt/riscv-musl/riscv64-linux-musl/lib",
    "-C", "target-feature=+crt-static"
]
linker = "riscv64-linux-musl-gcc"
```

编译命令也需要修改:

```
cargo build --release -Z build-std=core,std,panic_abort
```

在编写程序时，我们可以使用`#![no_main]` 关闭默认的`main` 函数。这可以减少一些不必要的系统调用，因为rust的默认实现会做一些保护措施，一上来就会调用奇怪的系统调用。



## 提示找不到运行时库的问题

运行时库在工具链中存在，但需要我们手动指定，-L命令用于指定搜索路径。

在工具链中不存在unwind.o这个库，但是rust所需要的符号可以在libgcc.a 这个库中找到，因此按照[创建符号链接](https://github.com/rust-lang/rust/issues/120655)的方法，可以解决这个问题。



https://github.com/m00nwtchr/oc2_hello_world 
https://stackoverflow.com/questions/74424444/how-to-build-a-rust-program-for-riscv64gc-unknown-linux-musl

https://www.reddit.com/r/rust/comments/17nxdc3/help_trying_to_build_for_riscv64gcunknownlinuxmusl/

https://github.com/rust-lang/rust/issues/120655 给出了解决方案



## 缺少浮点数相关的函数的问题

在创建线程的程序中，编译时会产生一系列符号缺失错误，原因是在这个target下这些函数没有被实现。解决方法是静态链接 c 库实现。

```
"-C", "link-arg=-lgcc"
```







https://www.cnblogs.com/fengyc/p/13665471.html