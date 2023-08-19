# PLIC

PLIC(platform-level interrupt controller)，**平台级中断控制器**。用来将外部的全局中断请求处理后转至中断目标。**PLIC理论上支持1023个外部中断源和15872个上下文**。

PLIC组成

- interrupt gateways：通常有多个，每个interrupt source对应一个
- PLICcore：负责interrupt priorittization and routing

中断源

- 每个interrupt source都会被赋予一个从1开始的无符号整数作为标识（*Interrupt Identifiers* ID）。0保留为“no interrupt”。该ID也作为在多个sources具有相同优先级是的选择条件：数值较小的优先于较大的。
- 每个interrupt source都会绑定一个与平台相关的优先级寄存器（符合WARL），数值不能为0。硬件也可以选择将优先级直接固定为硬连接。优先级寄存器中能够修改的bits组成的全部数值都必须被支持（例如寄存器中有2个bits可以读写，那么必须支持2x2=4种优先级）
  - 软件可以通过向寄存器中写全1/0再读出来的方式判断哪些bits硬连为0/1，哪些bits可写
- 每个interrupt source都被赋予一个*enable bit* IE，储存在平台相关的寄存器中（符合WARL）。当然IE bits也可以硬链接为0/1

中断目标

- 中断处理终端，通常为hart contexts，具体为一个RISCV CPU的特定privilege mode
- 并不是所有的hart context都是interrupt target。如果CPU不支持中断转移至低优先级（delegate），则低优先级的hart context不是interrupt target。
- PLIC产生的interrupt notification会标示在target的meip/heip/seip/ueip bits of mip/hip/sip/uip registers for M/H/S/U mode。只有支持delegate的处理器才会在对应lower privilege xip寄存器中置对应中断pending位
- PLIC不负责处理中断抢占（preempt）和嵌套（nest），由interrupt target处理上述问题。
- interrupt target对应平台相关的priority threshold寄存器（符合WARL），只有高于该threshold的active interrupt才会发送给对应target。threshold必须支持0，表示没有interrupt被mask；通常也需要支持max priority level，表示所有interrupt都会被mask



对于不同的处理器实现，上述描述中存在差异的地方在于中断目标，例如qemu与sifive u74-MC两个想比，qemu的硬件上下文是均匀的，即对与每个处理器核，都存在S/M两个mode，因此在设置相关的寄存器信息时，可以直接按照固定的偏移进行读写，但是对于u74-mc这种设计来说，其第一个核只有M mode，因此PLIC中硬件上下文的排布就不均匀，想比qemu偏移量就会变化。为了可以使得PLIC驱动可以同时兼容上面两种情况，我们在PLIC的初始化函数中要求用户提供当前机器上的硬件上下文信息。

```rust
#[derive(Debug)]
pub struct PLIC {
    base_addr: usize,
    privileges: [u8; HART_NUM],
}
```

对于qemu的初始化：

```rust
let privileges = [2;CPU_NUM];
let plic = PLIC::new(addr, &privileges);
```

对于u74-mc的初始化：

```rust
let mut privileges = [2u8;CPU_NUM];
// core 0 don't have S mode
privileges[0] = 1;
let plic = PLIC::new(addr, &privileges);
```

两者的差别在于第一个核的上下文只有一个，在PLIC内部，在设置外部中断时，会根据这个上下文计算出正确的偏移。



## Reference

plic：https://tinylab.org/riscv-irq-analysis-part2-interrupt-handling-plic/

https://blog.csdn.net/moonllz/article/details/52251788
