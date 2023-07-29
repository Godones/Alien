TRACE_EXE  := trace_exe
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
#IMG1 := tools/fs1.img

APPS_NAME := $(shell cd apps && ls -d */ | cut -d '/' -f 1)
VF2 ?=n
CV1811h ?=n
FEATURES :=
QEMU_ARGS :=
MEMORY_SIZE := 128M
img ?=fat32
SLAB ?=n
TALLOC ?=y
BUDDY ?=n

comma:= ,
empty:=
space:= $(empty) $(empty)

ifeq ($(GUI),y)
QEMU_ARGS += -device virtio-gpu-device \
			 -device virtio-keyboard-device \
			 -device virtio-mouse-device
else
QEMU_ARGS += -nographic
endif


ifeq ($(VF2),y)
FEATURES += vf2
else ifeq ($(CV1811h),y)
FEATURES += cv1811h
else
FEATURES += qemu
endif

ifeq ($(SLAB),y)
FEATURES += slab
else ifeq ($(TALLOC),y)
FEATURES += talloc
else ifeq ($(BUDDY),y)
FEATURES += buddy
endif


FEATURES := $(subst $(space),$(comma),$(FEATURES))

define boot_qemu
	qemu-system-riscv64 \
        -M virt $(1)\
        -bios $(BOOTLOADER) \
        -drive file=$(IMG),if=none,format=raw,id=x0 \
        -device virtio-blk-device,drive=x0 \
        -kernel  kernel-qemu\
        -$(QEMU_ARGS) \
        -smp $(SMP) -m $(MEMORY_SIZE) \
        -serial mon:stdio
endef

all:

install:
	@cargo install --git  https://github.com/os-module/elfinfo
	@#cd $(TRACE_EXE_PATH) && cargo build --release

build:compile


compile:
	cargo build --release -p boot --target riscv64gc-unknown-none-elf --features $(FEATURES)
	@(nm -n ${KERNEL_FILE} | $(TRACE_EXE) > kernel/src/trace/kernel_symbol.S)
	@#call trace_info
	cargo build --release -p boot --target riscv64gc-unknown-none-elf --features $(FEATURES)
	@#$(OBJCOPY) $(KERNEL_FILE) --strip-all -O binary $(KERNEL_BIN)
	@cp $(KERNEL_FILE) ./kernel-qemu

trace_info:
	@(nm -n ${KERNEL_FILE} | $(TRACE_EXE) > kernel/src/trace/kernel_symbol.S)

user:
	@cd apps && make all

sdcard:$(img) testelf user

run:install compile sdcard
	@echo qemu booot $(SMP)
	$(call boot_qemu)
	@#rm ./kernel-qemu


fake_run:
	$(call boot_qemu)



board:install compile
	@rust-objcopy --strip-all $(OUTPUT)/boot -O binary $(OUTPUT)/testos.bin
	@cp $(OUTPUT)/testos.bin  /home/godones/projects/tftpboot/
	@cp $(OUTPUT)/testos.bin ./alien.bin

vf2:board
	@mkimage -f ./tools/vf2.its ./alien-vf2.itb
	@#rm ./alien.bin
	@cp ./alien-vf2.itb /home/godones/projects/tftpboot/


cv1811h:board
	@mkimage -f ./tools/cv1811h.its ./alien-cv1811h.itb
	@#rm ./alien.bin
	@cp ./alien-cv1811h.itb /home/godones/projects/tftpboot/



f_test:
	qemu-system-riscv64 \
		-machine virt \
		-kernel kernel-qemu \
		-m 128M \
		-nographic \
		-smp 2 \
	    -drive file=./tools/sdcard.img,if=none,format=raw,id=x0  \
	    -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
	    -device virtio-net-device,netdev=net -netdev user,id=net

testelf:
	@#sudo mkdir /fat/ostest
	@#sudo cp test/* /fat/ostest -r
	@#sudo mkdir /fat/libc
	if [ -d "sdcard" ]; then \
		sudo cp sdcard/* /fat -r; \
		sudo cp sdcard/* /fat/bin -r;\
	fi
	#if [ -d "tools/siglibc" ]; then \
#		sudo cp tools/siglibc/build/* /fat/libc -r; \
#	fi
	@sync

dtb:
	$(call boot_qemu, -machine dumpdtb=riscv.dtb)
	@dtc -I dtb -O dts -o riscv.dts riscv.dtb
	@rm riscv.dtb

jh7110:
	@dtc -I dtb -o dts -o jh7110.dts ./tools/jh7110-visionfive-v2.dtb


SecondFile:
	#创建64MB大小空白文件
	@dd if=/dev/zero of=$(IMG1) bs=1M count=64

ZeroFile:
	#创建空白文件
	@dd if=/dev/zero of=$(IMG) bs=1M count=64

fat32:
	if [ -f "$(IMG)" ]; then \
		rm $(IMG); \
		touch $(IMG); \
	fi
	@sudo dd if=/dev/zero of=$(IMG) bs=1M count=256
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

gdb-server:
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

gdb-client:
	@riscv64-unknown-elf-gdb -ex 'file $(KERNEL_FILE)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'

kernel_asm:
	@riscv64-unknown-elf-objdump -d target/riscv64gc-unknown-none-elf/release/boot > kernel.asm
	@vim kernel.asm
	@rm kernel.asm


clean:
	@cargo clean
	@rm riscv.*
	@rm kernel-qemu
	@rm alien-*

.PHONY: all run clean fake_run

