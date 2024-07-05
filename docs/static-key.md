# static keys 

static key通过gcc的feature和代码修补技术，可以在内核性能敏感路径上包含很少执行的函数，同时不影响性能。产生static key的动机是内核的trace point功能，这个功能使用条件分支进行判断，虽然判断的开销很小，但是因为trace point需要检查全局变量，这些全局变量需要在多核之间同步，当这些全局变量增多时，会给缓存带来较大的压力。这些trace point通常是不会被打开的，最好的情况是在不开起的情况下不会去做判断操作。linux使用的这种修补机制是**跳转标签修补**



**asm goto**：gcc 的一项特性，允许跳转到一个标签上

rust种对应的特性为`#[feature(asm_goto)]`。



| default enable | likely/unlikely | JumpType |
| -------------- | --------------- | -------- |
| false          | likely          | JMP      |
| false          | unlikely        | NOP      |
| true           | likely          | NOP      |
| true           | unlikely        | JMP      |



`likely` 意味着说大多数情况下为true，而`arch_static_branch` 默认是走`nop`路线并且返回false，`arch_static_branch_jump`默认走`jump`路线返回true。



```c
#define static_branch_likely(x)							\
({										\
	bool branch;								\
	if (__builtin_types_compatible_p(typeof(*x), struct static_key_true))	\
		branch = !arch_static_branch(&(x)->key, true);			\
	else if (__builtin_types_compatible_p(typeof(*x), struct static_key_false)) \
		branch = !arch_static_branch_jump(&(x)->key, true);		\
	else									\
		branch = ____wrong_branch_error();				\
	likely_notrace(branch);								\
})

#define static_branch_unlikely(x)						\
({										\
	bool branch;								\
	if (__builtin_types_compatible_p(typeof(*x), struct static_key_true))	\
		branch = arch_static_branch_jump(&(x)->key, false);		\
	else if (__builtin_types_compatible_p(typeof(*x), struct static_key_false)) \
		branch = arch_static_branch(&(x)->key, false);			\
	else									\
		branch = ____wrong_branch_error();				\
	unlikely_notrace(branch);							\
})
```





## reference

[linux doc for static keys](https://docs.kernel.org/staging/static-keys.html) 

[gcc asm goto rfc](https://gcc.gnu.org/legacy-ml/gcc-patches/2009-07/msg01556.html)

[rust asm goto ](https://github.com/rust-lang/rust/issues/119364)  rust对asm goto 的支持

[内核基础设施——static_key](https://linux.laoqinren.net/kernel/static_key/) 讲解了一些细节，大概了解是如何产生nop指令以及需要保存哪些信息

 [statiic keys in ebpf](https://lpc.events/event/17/contributions/1608/attachments/1278/2578/bpf-static-keys.pdf)

https://tinylab.org/riscv-jump-label-part3/ 比较完整的在riscv架构下的实现介绍