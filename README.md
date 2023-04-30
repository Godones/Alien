# Alien

A simple operating system implemented in rust. The purpose is to explore how to use modules to build a complete os,
so the system is composed of a series of independent modules. At present, the system already supports user-mode programs
and some simple functions.

## Modules

`pci` ：pci driver to detect devices on the bus

`rtc` ：rtc driver to get time

`page-table `： page table to manage virtual memory

`simple-bitmap`： a simple bitmap to manage frames

`gmanager`: a simple allocator to manage `process`/ `fd` and so on

`rvfs `: a vfs framework like linux, it can support multiple file systems

`fat32-vfs`: a disk file system, it can support fat32

`jammdb `: a key-value database, it can support `dbfs`

`dbfs2 `:  a disk file system, it is based on `jammdb`

`trace_lib `: stack trace library

`preprint ` : a simple print library

`rslab `: slab allocator like linux

`syscall-table `: A tool to automatically collect syscalls

`dbop`: Make database functions available to users as system calls

`plic`: riscv plic driver

`uart`: uart driver, it supports interrupt

Other modules are not listed here, you can find them in the cargo.toml file.

## Run

1. install qemu 7.0.0
2. install rust nightly

```
make run LOG=WARN img=fat32
```

## TODO

- [ ] Thread/Mutil-core
- [x] full vfs
- [x] fat32
- [x] dbfs
- [ ] Mutex
- [x] sleep task queue
- [x] uart task queue
- [ ] block driver task queue
- [x] a simple shell
- [x] memory management
- [x] process management