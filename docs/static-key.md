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



经过尝试，在rust中实现Linux中的static keys机制不可行：

1. rust不支持C语言中的label机制
2. C的static keys实现中gcc编译器可以识别函数即可能返回true，也可能返回false，但rust的编译导致一个函数不能这种情况。当然这理论上也跟rust不支持label支持一样

直接模仿Linux的实现不可行，可以试试其它途径。

## Naked Function

motivation： 

> static keys的核心是通过修改指令来实现不同的路径选择，如果我们观察一个默认返回false的函数，这种函数内部的指令应该非常简单，是否可以通过直接修改函数内部的指令做到动态修改其返回值呢？

答案是可行的。在rust中，有一种特殊的函数裸函数`naked function`， 这种函数gcc中也有。裸函数早已成为编译器的一个特性。这些函数通常在各方面都定义为普通函数，只是编译器不会发出函数序言和结尾。

https://rust-lang.github.io/rfcs/2972-constrained-naked.html

也就是说我们可以定义一个和普通函数一样的函数，其内部完全由我们自定义的汇编实现。表面上其它函数调用是就像一个正常函数一样，但是这个函数的内部不再有多余的栈保存和栈恢复的代码生成。



### RFC的解释

由于裸函数没有序言，任何使用堆栈的天真尝试都可能产生无效代码。这当然包括局部变量。但这也可能包括尝试引用可能放在堆栈上的函数参数。这就是为什么裸函数可能只包含一个`asm!()`语句的原因。

此外，由于许多平台将返回地址存储在堆栈中，因此语句有责任`asm!()`以适当的方式返回。这就是为什么`options(noreturn)`需要该选项的原因。

任何尝试使用函数参数（即使作为操作数）的行为都可能导致堆栈访问或修改。同样，任何寄存器操作数都可能导致编译器尝试在堆栈上保留寄存器。由于该函数没有序言，因此这是有问题的。为了避免这个问题，我们干脆拒绝在 Rust 中使用任何函数参数。

如果这就是故事的结局，那么裸函数就没什么用了。为了重新启用对函数参数的访问，编译器确保语句中寄存器的初始状态`asm!()`符合函数的调用约定。这允许通过调用约定手工编写汇编代码访问函数参数。由于`extern "Rust"`调用约定未定义，因此不鼓励使用它，而应指定替代的、定义明确的调用约定。同样，由于语句`asm!()`可以通过调用约定访问函数参数，因此参数本身应该是 FFI 安全的，以确保可以从汇编中可靠地访问它们。

由于裸函数严重依赖调用约定，内联这些函数会使代码生成变得极其困难。因此，我们不允许内联。

由于`const`和`sym`操作数既不修改堆栈也不修改寄存器，因此允许使用它们。



### 实现

有了这种函数，我们可以实现下面的函数:

```rust
#[naked_function::naked]
pub unsafe extern "C" fn is_true() -> bool {
    asm!("li a0, 1", "ret",)
}
```

对比下面这种函数：

```rust
pub fn is_true()->bool{
    return true
}
```

两者的功能是一致的，但是下面这种函数会被编译器优化掉，而上面的函数不会。这给了我们机会通过修改第一条指令的立即数，就可以更改这个函数的行为。



这种实现相比Linux中的实现，是存在一些缺陷的：

1. 首先这种方法不能减少指令数量，gcc可以判断函数默认返回值并做代码优化，这会减少不必要的测试跳转指令，但rust的这种方法并不能减少测试跳转指令
   1. 在riscv编译的结果显示，对比直接进行判断的实现，这种调用裸函数的方法会增加1条额外的指令
2. 这种方法唯一的优势在于，其可以减少内存的访问，以及不必要的同步指令，因为直接读取原子变量判断的过程会生成内存访问和同步指令
   1. 我们设想这种方法减少的内存访问和同步开销可以抵消指令增加带来的开销
   2. 同时，这种方法还能减少缓存的冲突，内核中大量的变量会迅速填充cpu缓存，导致之前读取的变量信息被逐出，需要再一次从内存读取。这应该可以进一步减少其开销



## 测试

我们在riscv的开发板进行测试，评估读取原子变量进行判断和使用裸函数进行判断的性能数据。

单纯地评估两种实现的差异并没有意义，我们模拟了缓存大量数据填充的影响。





### 单个变量的影响

```rust
#[naked_function::naked]
pub unsafe extern "C" fn is_false() -> bool {
    asm!("li a0, 1", "ret",)
}

unsafe fn test_static_keys() -> usize {
    let mut count = 0;
    let now = read_time_us();
    for _ in 0..LOOP {
        if is_false() {
            count += 1;
        }
        maybe_modify();
        if is_false() {
            count += 1;
        }
    }
    let end = read_time_us();
    println!("test_static_keys: {}us", end - now);
    println!("test_static_keys: {}", count);
    end - now
}

fn test_static_atomic() -> usize {
    let mut count = 0;
    let now = read_time_us();
    for _ in 0..LOOP {
        if FLAG.load(core::sync::atomic::Ordering::SeqCst) {
            count += 1;
        }
        maybe_modify();
        if FLAG.load(core::sync::atomic::Ordering::SeqCst) {
            count += 1;
        }
    }
    let end = read_time_us();
    println!("test_atomic: {}us", end - now);
    println!("test_atomic: {}", count);
    end - now
}
```

这个测试中只会对单个变量进行判断，在每一次循环中，`maybe_modify` 会执行填充数据的操作，以模拟填充缓存的行为。

```rust
pub static mut DATA_CACHE: [DataCache; 32] = [DataCache::new(); 32];

fn maybe_modify() {
    let time = read_time();
    if time < 1000 {
        FLAG.store(false, core::sync::atomic::Ordering::SeqCst);
        FLAGS[time].set_val(1);
    }
    unsafe {
        let cache = &mut DATA_CACHE[hart_id()];
        cache.fill((time % 255) as u8);
    }
}
```

每个DataCache的大小为2MB，这应该可以将一级缓存和二级缓存填满。

测试的结果如下：

```
单个static atomic的影响:
atomic		static keys
1863180us	1603466us	13.94
1862927us	1603440us	13.93
1863036us	1603458us	13.93

avg = 1863047us

static keys	atomic
1605054us	1861353us	13.77
1604889us	1861388us	13.78
1605057us	1605057us	13.77

avg = 1605000us
=> 13.85%
```

这种情况下使用原子变量的开销增加了13.85%.

1. 每个循环都执行了缓存填充的操作，导致了变量需要从下一级缓存或者内存中读取



### 多个变量的影响

由于内核中存在大量的分支判断，因此多个变量的测试更加符合内核中的情况。

#### 集中判断

```rust
fn test_mass_static_atomic(cpu: usize) {
    let mut count = 0;
    let now = read_time_us();
    for i in 0..100 {
        for index in 0..1000 {
            if FLAGS[index].is_true() {
                count += 1;
            }
        }
        maybe_modify();
        for index in 0..1000 {
            if FLAGS[index].is_true() {
                count += 1;
            }
        }
    }
    let end1 = read_time_us() - now;
}

fn test_mass_static_keys(cpu: usize) {
    let mut count = 0;
    let now = read_time_us();
    for i in 0..100 {
        for _ in 0..1000 {
            unsafe {
                if is_false() {
                    count += 1;
                }
            }
        }
        maybe_modify();
        for _ in 0..1000 {
            unsafe {
                if is_false() {
                    count += 1;
                }
            }
        }
    }
    let end2 = read_time_us() - now;
}
```

这个测试每个循环会读取所有变量进行判断，然后填充一次缓存，再执行一次判断。这些变量都是缓存行大小。

测试结果如下:

```
atomic		static keys
170719us	162238us	4.97
170991us	162221us	5.13
171043us	162219us	5.16

avg = 170917us

static keys	atomic
164091us	169222us	3.03
164428us	169212us	2.83
164389us	169222us	2.86

avg = 164302us
=> 3.8%
```

这里使用变量进行判断带来了3.8%的开销，与第一种相比，这里的开销减少了许多

1. 由于每次都是先把所有变量都判断完，再进行填充缓存的操作，相比第一种每次读取变量后都进行填充操作，这减少了这些变量的缓存失效

这种在内核中其实是一种理想情况，因为不是所有变量都是一次性读取的。



#### 分散判断

```rust
fn test_mass_static_atomic(cpu: usize) {
    let mut count = 0;
    let now = read_time_us();
    for i in 0..10 {
        for index in 0..100 {
            if FLAGS[index].is_true() {
                count += 1;
            }
            maybe_modify();
        }
    }
    let end1 = read_time_us() - now;
    println!("test_atomic: {}us", end1);
    println!("test_atomic: {}", count);
    TIME_ATOMIC.store(end1 as usize, atomic::Ordering::SeqCst);
}
fn test_mass_static_keys(cpu: usize) {
    let mut count = 0;
    let now = read_time_us();
    for i in 0..10 {
        for _ in 0..100 {
            unsafe {
                if is_false() {
                    count += 1;
                }
            }
            maybe_modify();
        }
    }
    let end2 = read_time_us() - now;
    println!("test_static_keys: {}us", end2);
    println!("test_static_keys: {}", count);
    TIME_KEYS.store(end2 as usize, atomic::Ordering::SeqCst);
}

```

这个测试与单个变量的非常相似，不过这里会读取所有的变量进行判断。而且与上一种集中进行判断不同，这里每进行一次判断都会执行一次填充操作。理论上来书，这种与内核的情形更为相似，因为内核中不同变量读取之间可能已经经过了许多操作，缓存早已被冲刷掉。

这种情况的测试结果如下:

```rust
atomic	static keys
2035588us	1772564us	12.92
2035594us	1772585us	12.92
2035034us	1772570us	12.90

avg = 2035405us

static keys	atomic
1605288us	1603604us	0.10
1605217us	1603608us	0.10
1605076us	1603621us	0.09

avg = 1605193us
=> 21.13%
```

测试结果表面这种情况下使用变量进行判断带来近21%的开销，比第一种还要严重。

1. 与第一种相比，这里在填充缓存后读取的变量是不同的，这些变量有很大概率没有在缓存中，可能导致需要到内存中读取
2. 由于这些变量的数量很多，下一个读到的变量很大概率会被刷出缓存中



## reference

[linux doc for static keys](https://docs.kernel.org/staging/static-keys.html) 

[gcc asm goto rfc](https://gcc.gnu.org/legacy-ml/gcc-patches/2009-07/msg01556.html)

[rust asm goto ](https://github.com/rust-lang/rust/issues/119364)  rust对asm goto 的支持

[内核基础设施——static_key](https://linux.laoqinren.net/kernel/static_key/) 讲解了一些细节，大概了解是如何产生nop指令以及需要保存哪些信息

 [statiic keys in ebpf](https://lpc.events/event/17/contributions/1608/attachments/1278/2578/bpf-static-keys.pdf)

https://tinylab.org/riscv-jump-label-part3/ 比较完整的在riscv架构下的实现介绍