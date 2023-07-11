## 最高优先级

### 线程(1week)

- [x] 完整线程支持
  - 进程和线程使用同一个数据结构
  - 根据clone的参数进行具体打初始化
- [x] 多核调度
  - 全局+Per-CPU
- [x] 线程相关的初始化
  - tls

### 动态链接(3days)

- [ ] elf文件的处理
  - 处理elf文件类型，并根据类型初始化相关的数据结构
- [ ] 软链接处理 ---> use dbfs

### 系统调用(one week)

- [ ] more syscall support

`ioctl`: https://zhuanlan.zhihu.com/p/478259733

`writev`: https://man7.org/linux/man-pages/man2/writev.2.html 

`piep`：https://blog.csdn.net/oguro/article/details/53841949

`fcntl`：https://man7.org/linux/man-pages/man2/fcntl64.2.html

`futex`:https://man7.org/linux/man-pages/man2/futex.2.html   http://linuxperf.com/?p=23