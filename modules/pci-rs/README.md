# PCI Driver
A platform-agnostic PCI bus management and configuration access library forked from [robigalia/pci](https://gitlab.com/robigalia/pci).

## Support status

Supported features:
* PCI capabilities listing
* Access the configuration space with I/O functions for x86_64
* Access the configuration space with CAM and ECAM for RISCV
* x86_64 on Qemu is supported
* x86_64 on PC is supported
* RISCV on Qemu is supported
* RISCV fu740 board is supported

## Example

```
cd examples/riscv
make run
```
## Reference
* Linux/U-Boot source code
* [PCIE Specifications](https://pcisig.com/specifications)
* [Linux PCI驱动框架分析](https://mp.weixin.qq.com/s?__biz=MzU1MDkzMzQzNQ==&mid=2247484340&idx=1&sn=fa20f4ef93cd332ad7ef0f58c04fb55c)
