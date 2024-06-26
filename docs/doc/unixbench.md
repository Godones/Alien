# Unixbench

记录`unixbench`测试中出现的错误以及解决方法

## bug1

时钟中断被忽略的问题 

在运行`unixbench`的测试时，发现程序会在中途卡住不动，打开debug信息后，发现程序是在`./looper 20 ./multi.sh 1` 中暂停的，查看源代码发现这个是一个死循环。但是内核中的时钟中断应该让其被切换掉。因此在时钟中断的处理函数中打印了信息，再运行发现程序在一开始都还有频繁的时钟中断发生，到后面后就不再发生时钟中断了。

在异常处理中，我们获取了当前的中断状态信息，发现依然是正确的。那问题应该就是在某个时刻，内核在处理某个事情时，关闭了中断，但是此时时钟中断来了，由于全局中断被关闭，时钟中断将被忽略不处理，这时就无法设置下一个时钟，造成后面就不会再发生时钟中断了。

知道大概原因，我们查看了riscv手册，发现`sip`寄存器保存了中断待定信息，通过查看这个寄存器可以知道还有什么中断没有被处理。

在用户态与内核态切换以及任务切换中我们增加了对于时钟中断的检查，当发现其没有被处理时，我们就手动添加下一次的时钟。

![image-20230723233211257](assert/image-20230723233211257.png)
