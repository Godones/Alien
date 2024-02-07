# Ext文件系统支持

在之前的系统中，我们对磁盘文件系统只支持FAT32，为了使得文件系统具备更多的功能，这段时间我们移植了ext系列的文件。

## lwext4 ？

The main goal of the lwext4 project is to provide ext2/3/4 filesystem for microcontrollers. It may be an interesting alternative for traditional MCU filesystem libraries (mostly based on FAT32). Library has some cool and unique features in microcontrollers world:

- directory indexing - fast file find and list operations
- extents - fast big file truncate
- journaling transactions & recovery - power loss resistance

Lwext4 is an excellent choice for SD/MMC card, USB flash drive or any other wear leveled memory types. However it is not good for raw flash devices.

You can find more information from [lwext4](https://github.com/gkostka/lwext4).

## How to do it ?

lwext是一个C语言编写的库，为了在rust中使用它，我们需要生成C语言绑定。我们使用[bindgen](https://github.com/rust-lang/rust-bindgen)来完成这个工作。在此之前，需要让lwext4支持`no_std`环境和`riscv`平台，由于lwext4是为嵌入式环境实现，具有良好的可移植性，因此只需要参考已有的几个平台，并添加新平台的支持，这需要对makefile进行简单的修改以使用musl工具链并去除标准库的依赖。

[lwext4-c](https://github.com/os-module/lwext4-c) 是这部分的具体实现。

有了c实现的支持，我们只需要在rust中生成相关的头文件以及静态库。在做这部分之前，我们首先查看了一下`crates.io`中是否已有相关的实现，幸运的是，2年前已经有一个实现[lwext4](https://github.com/djdisodo/lwext4), 在简单阅读了其实现之后，我们打算参考其实现重新编写，因为其已经缺乏维护，并且不包含对`no_std`环境的支持。这个已有的实现给予我们很好的想法。

在[lwext4](https://github.com/os-module/lwext4)中，为了更好的扩展性，我们添加了三个`crate`:

- lwext4-sys
- lwext4-rs
- lwext4-mkfs

lwext4-sys是生成绑定的库，其只是简单地导出生成的c绑定。

lwext4-rs使用lwext-sys，并使用更多的rust实现，尽可能地降低unsafe代码块出现。我们实现了`lwext4`中绝大部分功能。并提供了详细的例子和简单的测试，从而确保使用者可以更轻松地理解和使用。

lwext-mkfs只是对lwext-rs库中mkfs功能简单的封装，使用者可以使用其格式化磁盘，但更推荐直接使用`mkfs.ext`进行格式化。

## How to use ？

因为我们内核中已经引入了一个简单`VFS`，所以我们也参考之前的文件系统实现，对`lwext-rs`进行封装并实现`VFS`的接口。这部分的工作有了之前的参考而且本身对lwext4的封装也比较完善，因此实现起来较为轻松。

[lwext4-vfs](https://github.com/os-module/rvfs/tree/main/lwext4-vfs)提供了部分功能的实现，满足现在内核的需求。

后期我们仍需要对其`VFS`进行完善并加入更多功能。

实现`VFS`接口后，将其引入内核的做法和FAT32的类似。因为通常我们只挂载一块磁盘，因此我们使用`cfg`对磁盘文件系统的使用进行选择。

在`MakeFile`中，我们添加了ext磁盘格式的支持。

在lwext4的c实现配置中，我们开启了debug输出，所以其会依赖`printf/fflush/stdout` 进行输出，除此之外，其还依赖几个函数:

1. `malloc` / `free` / `calloc` / `realloc`
2. `strcmp` / `strcpy` / `strncmp`
3. `qsort`

为了处理这几个函数依赖，我们手动提供这些实现，也可以使用一些已有的实现。[tinyrlibc](https://github.com/rust-embedded-community/tinyrlibc) 库提供了1和2的实现。为了实现`printf`，可以参考[prinf_compat](https://docs.rs/printf-compat/0.1.1/printf_compat/). [c-ward](https://github.com/sunfishcode/c-ward)提供了qsort的实现，可以直接从这里复制。 最后，我们需要实现的只是`fflush`和`stdout`。 通常，我们只需要将这两个实现为空函数即可。

```rust
#[no_mangle]
static stdout: usize = 0;

#[no_mangle]
extern "C" fn fflush(file: *mut c_void) -> c_int{
    assert!(file.is_null());
    0
}
```

