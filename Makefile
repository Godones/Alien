.PHONY: all run clean

TARGET      := riscv64gc-unknown-none-elf
KERNEL_FILE := boot/target/$(TARGET)/release/boot
DEBUG_FILE  ?= $(KERNEL_FILE)
KERNEL_ENTRY_PA := 0x80200000
OBJDUMP     := rust-objdump --arch-name=riscv64
OBJCOPY     := rust-objcopy --binary-architecture=riscv64
BOOTLOADER  := ./boot/rustsbi-qemu.bin
BOOTLOADER  := default
KERNEL_BIN  := $(KERNEL_FILE).bin
IMG := tools/fs.img
SMP :=1

IMG1 := tools/fs1.img

FS_TYPE := fat32

define boot_qemu
	qemu-system-riscv64 \
        -M virt $(1)\
        -bios $(BOOTLOADER) \
        -device loader,file=kernel-qemu,addr=$(KERNEL_ENTRY_PA) \
        -drive file=$(IMG),if=none,format=raw,id=x0 \
        -device virtio-blk-device,drive=x0 \
	  	-drive file=$(IMG1),if=none,format=raw,id=x1 \
		-device virtio-blk-device,drive=x1 \
        -nographic \
        -kernel  kernel-qemu\
        -smp $(SMP) -m 128M
endef


all:run

compile:
	@cd boot && cargo build --release
	@$(OBJCOPY) $(KERNEL_FILE) --strip-all -O binary $(KERNEL_BIN)
	@cp $(KERNEL_FILE) ./kernel-qemu


user:
	@cd apps && make

build:compile

run:compile $(img) user
	$(call boot_qemu)
	@rm ./kernel-qemu


dtb:
	$(call boot_qemu, -machine dumpdtb=riscv.dtb)
	@dtc -I dtb -O dts -o riscv.dts riscv.dtb
	@rm riscv.dtb

ZeroFile:
	#创建空白文件
	@dd if=/dev/zero of=$(IMG) bs=1M count=64

fat32:
	#创建64MB大小的fat32文件系统
	@sudo chmod 777 $(IMG)
	@sudo dd if=/dev/zero of=$(IMG) bs=512 count=131072
	@sudo mkfs.fat -F 32 $(IMG)
	@if mountpoint -q /fat; then \
		sudo umount /fat; \
	fi
	@sudo mount $(IMG) /fat
	@sync


img-hex:
	@hexdump $(IMG) > test.hex
	@cat test.hex


gdb: compile $(img) user
	@qemu-system-riscv64 \
            -M virt $(1)\
            -bios $(BOOTLOADER) \
            -device loader,file=kernel-qemu,addr=$(KERNEL_ENTRY_PA) \
            -drive file=$(IMG),if=none,format=raw,id=x0 \
            -device virtio-blk-device,drive=x0 \
            -nographic \
            -kernel  kernel-qemu\
            -smp $(SMP) -m 128M \
            -s -S

debug: compile $(img) user
	@tmux new-session -d \
		"qemu-system-riscv64 -machine virt -nographic -bios $(BOOTLOADER) -device loader,file=kernel-qemu,addr=$(KERNEL_ENTRY_PA) \
		-drive file=$(IMG),if=none,format=raw,id=x0  -device virtio-blk-device,drive=x0 -smp 1 -m 128M -s -S" && \
		tmux split-window -h "riscv64-unknown-elf-gdb -ex 'file $(KERNEL_FILE)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'" && \
		tmux -2 attach-session -d
fmt:
	cd boot && cargo fmt
	cd apps && make fmt
	cd kernel && cargo fmt
	cd userlib && cargo fmt
asm:compile
	@riscv64-unknown-elf-objdump -d target/riscv64gc-unknown-none-elf/release/kernel > kernel.asm
	@lvim kernel.asm
	@rm kernel.asm
clean:
	@cargo clean
	@rm riscv.*

