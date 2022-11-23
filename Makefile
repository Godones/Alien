.PHONY: all run clean

TARGET      := riscv64gc-unknown-none-elf
KERNEL_FILE := target/$(TARGET)/release/kernel
DEBUG_FILE  ?= $(KERNEL_FILE)
KERNEL_ENTRY_PA := 0x8020000
OBJDUMP     := rust-objdump --arch-name=riscv64
OBJCOPY     := rust-objcopy --binary-architecture=riscv64
BOOTLOADER  := ./bootloader/rustsbi-qemu.bin
KERNEL_BIN  := $(KERNEL_FILE).bin

all:
	@#rm  kernel-qemu
	@cargo build --release -p kernel
	@$(OBJCOPY) $(KERNEL_FILE) --strip-all -O binary $(KERNEL_BIN)
	@cp $(KERNEL_FILE) ./kernel-qemu


build:all

run:all
	@qemu-system-riscv64 \
    -machine virt \
    -bios $(BOOTLOADER) \
    -device loader,file=kernel-qemu,addr=$(KERNEL_ENTRY_PA) \
    -drive file=dependence/fat32.img,if=none,format=raw,id=x0 \
    -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
    -nographic \
    -smp 1 -m 2G

debug: build
	@tmux new-session -d \
		"qemu-system-riscv64 -machine virt -nographic -bios $(BOOTLOADER) -device loader,file=kernel-qemu,addr=$(KERNEL_ENTRY_PA) -s -S" && \
		tmux split-window -h "riscv64-unknown-elf-gdb -ex 'file $(KERNEL_FILE)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'" && \
		tmux -2 attach-session -d

asm:all
	@riscv64-unknown-elf-objdump -d target/riscv64gc-unknown-none-elf/release/kernel > kernel.asm
clean:
	@rm kernel-qemu
	@rm $(KERNEL_FILE)
