
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
SMP ?= 4
GUI ?=n
IMG1 := tools/fs1.img

APPS_NAME := $(shell cd apps && ls -d */ | cut -d '/' -f 1)


QEMU_ARGS :=


ifeq ($(GUI),y)
QEMU_ARGS += -device virtio-gpu-device \
			 -device virtio-keyboard-device \
			 -device virtio-mouse-device
else
QEMU_ARGS += -nographic
endif

define boot_qemu
	qemu-system-riscv64 \
        -M virt $(1)\
        -bios $(BOOTLOADER) \
        -device loader,file=kernel-qemu,addr=$(KERNEL_ENTRY_PA) \
        -drive file=$(IMG),if=none,format=raw,id=x0 \
        -device virtio-blk-device,drive=x0 \
        -kernel  kernel-qemu\
        -$(QEMU_ARGS) \
        -smp $(SMP) -m 256M \
        -serial mon:stdio
endef

install:
	@#cargo install --git  https://github.com/os-module/elfinfo
	@cd $(TRACE_EXE_PATH) && cargo build --release

all:run

compile:
	cargo build --release -p boot --target riscv64gc-unknown-none-elf
	@(nm -n ${KERNEL_FILE} | $(TRACE_EXE) > kernel/src/trace/kernel_symbol.S)
	@#call trace_info
	@cargo build --release -p boot --target riscv64gc-unknown-none-elf
	@$(OBJCOPY) $(KERNEL_FILE) --strip-all -O binary $(KERNEL_BIN)
	@cp $(KERNEL_FILE) ./kernel-qemu

trace_info:
	@(nm -n ${KERNEL_FILE} | $(TRACE_EXE) > kernel/src/trace/kernel_symbol.S)

user:
	@cd apps && make all



build:compile


run:install compile $(img) user testelf
	@echo qemu booot $(SMP)
	$(call boot_qemu)
	@#rm ./kernel-qemu


fake_run:
	@$(call boot_qemu)

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
	#创建fat32文件系统
	@sudo dd if=/dev/zero of=$(IMG) bs=1M count=64
	@sudo chmod 777 $(IMG)
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


gdb: compile $(img) user  testelf
	@qemu-system-riscv64 \
            -M virt $(1)\
            -bios $(BOOTLOADER) \
            -device loader,file=kernel-qemu,addr=$(KERNEL_ENTRY_PA) \
            -drive file=$(IMG),if=none,format=raw,id=x0 \
            -device virtio-blk-device,drive=x0 \
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


asm:
	@riscv64-unknown-elf-objdump -d target/riscv64gc-unknown-none-elf/release/boot > kernel.asm
	@vim kernel.asm
	@rm kernel.asm



clean:
	@cargo clean
	@rm riscv.*


.PHONY: all run clean fake_run

