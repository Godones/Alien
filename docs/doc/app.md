## 对app和Mstd的修改内容

+ 在app/tests下加入了[thread_create.rs](../../apps/tests/src/thread_create.rs)，其中的thread_test1()调用了两个Mstd中的系统调用thread_create(), gettid()(也就是sys_clone()和sys_gettid())


+ 修改系统调用号 220: SYSCALL_FORK -> SYSCALL_CLONE 以及syscall_clone的参数
+ 加入系统调用号 178: SYSCALL_GETTID 以及对应的系统调用

```rust
    // sys_gettid 返回当前正在执行的进程的tid
    fn sys_gettid() -> isize 
```
+ 修改了Mstd::process中fork的实现，改为调用sys_clone()
+ 在Mstd::thread中加入gettid()、create_thread()接口

+ 加入系统调用号 95: SYSCALL_WAITID, 但目前还没有加入syscall_waitid的参数，以及thread_join去调用该系统调用

```rust
    // sys_waitid提供了对要等待的子进程更精确的控制
    // id_type: 等待的子进程id号的类型
    // id: 要等待的子进程的id号，类型由id_type控制
    // infop: 函数成功返回后，infop所指向的siginfo_ti结构将会被填充。siginfo_ti结构中包括pid，uid等信息
    // options: 决定当等待的子进程的状态发生何种改变时函数才会返回。
    fn sys_waitid(id_type: flag, id: usize, infop: *siginfo_t, options: isize) -> isize;
```

[与sys_waitid相关flag和option的定义](https://man7.org/linux/man-pages/man2/waitpid.2.html)
