# Isolation in os subsystem



## 1. Introduction



## 2. Implementation

   
## 3. Run
- rust
- riscv64-linux-musl-gcc
- git submodule update --init --recursive

```
make run
```
```
make build        # build kernel
make sdcard       # build all domains and user app
make initrd       # build initrd (choose static busybox)
```
## Reference
[git submodule](https://iphysresearch.github.io/blog/post/programing/git/git_submodule/)