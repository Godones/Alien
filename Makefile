.PHONY: all run clean

TRACE_EXE_PATH:= ../os-module/elfinfo/
TRACE_EXE  := ../os-module/elfinfo/target/release/trace_exe
TARGET      := riscv64gc-unknown-none-elf
OUTPUT := target/$(TARGET)/release/
KERNEL_FILE := $(OUTPUT)/boot
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
APPS_NAME := $(shell cd apps && ls -d */ | cut -d '/' -f 1)

define boot_qemu
	qemu-system-riscv64 \
        -M virt $(1)\
        -bios $(BOOTLOADER) \
        -device loader,file=kernel-qemu,addr=$(KERNEL_ENTRY_PA) \
        -drive file=$(IMG),if=none,format=raw,id=x0 \
        -device virtio-blk-device,drive=x0 \
        -nographic \
        -kernel  kernel-qemu\
        -smp $(SMP) -m 256M \
        -serial mon:stdio
endef

install:
	@#cargo install --git  https://github.com/os-module/elfinfo
	@cd $(TRACE_EXE_PATH) && cargo build --release

all:run

compile:
	@cd boot && cargo build --release
	@(nm -n ${KERNEL_FILE} | $(TRACE_EXE) > kernel/src/trace/kernel_symbol.S)
	@#call trace_info
	@cd boot && cargo build --release
	@$(OBJCOPY) $(KERNEL_FILE) --strip-all -O binary $(KERNEL_BIN)
	@cp $(KERNEL_FILE) ./kernel-qemu

trace_info:
	@(nm -n ${KERNEL_FILE} | $(TRACE_EXE) > kernel/src/trace/kernel_symbol.S)

user:
	@cd apps && make all



build:compile


run:install compile $(img) user testelf
	$(call boot_qemu)
	@#rm ./kernel-qemu


test:install compile $(img) SecondFile testelf
	$(call boot_qemu)

testelf:
	@sudo mkdir /fat/ostest
	@sudo cp test/* /fat/ostest -r
	@sync

dtb:
	$(call boot_qemu, -machine dumpdtb=riscv.dtb)
	@dtc -I dtb -O dts -o riscv.dts riscv.dtb
	@rm riscv.dtb

SecondFile:
	#创建64MB大小空白文件
	@dd if=/dev/zero of=$(IMG1) bs=1M count=64

ZeroFile:
	#创建空白文件
	@dd if=/dev/zero of=$(IMG) bs=1M count=64

fat32:
	@#rm -rf ./tools/fs.img
	#创建64MB大小的fat32文件系统
	@sudo chmod 777 $(IMG)
	@sudo dd if=/dev/zero of=$(IMG) bs=1M count=64
	@sudo mkfs.fat -F 32 $(IMG)
	@if mountpoint -q /fat; then \
		sudo umount /fat; \
	fi
	@sudo mount $(IMG) /fat
	@sudo cp tools/f1.txt /fat
	@sudo mkdir /fat/folder
	@sudo cp tools/f1.txt /fat/folder
	@sync


img-hex:
	@hexdump $(IMG) > test.hex
	@cat test.hex


gdb: compile $(img)  user SecondFile testelf
	@qemu-system-riscv64 \
            -M virt $(1)\
            -bios $(BOOTLOADER) \
            -device loader,file=kernel-qemu,addr=$(KERNEL_ENTRY_PA) \
            -drive file=$(IMG),if=none,format=raw,id=x0 \
            -device virtio-blk-device,drive=x0 \
			-drive file=$(IMG1),if=none,format=raw,id=x1 \
			-device virtio-blk-device,drive=x1 \
            -nographic \
            -kernel  kernel-qemu\
            -smp 1 -m 128M \
            -s -S

debug: compile $(img) user
	@tmux new-session -d \
		"qemu-system-riscv64 -machine virt -nographic -bios $(BOOTLOADER) -device loader,file=kernel-qemu,addr=$(KERNEL_ENTRY_PA) \
		-drive file=$(IMG),if=none,format=raw,id=x0  -device virtio-blk-device,drive=x0 -smp 1 -m 128M -s -S" && \
		tmux split-window -h "riscv64-unknown-elf-gdb -ex 'file $(KERNEL_FILE)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'" && \
		tmux -2 attach-session -d
fmt:
	@cd boot && cargo fmt
	@cd apps && make fmt
	@cd kernel && cargo fmt
	@cd userlib && cargo fmt
	@cd modules && make fmt
asm:
	@riscv64-unknown-elf-objdump -d target/riscv64gc-unknown-none-elf/release/boot > kernel.asm
	@vim kernel.asm
	@rm kernel.asm



clean:
	@cd boot && cargo clean
	@cd apps && make clean
	@rm riscv.*

