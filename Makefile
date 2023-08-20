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
IMG := tools/sdcard.img
SMP ?= 1
GUI ?=n
NET ?=y
#IMG1 := tools/fs1.img

APPS_NAME := $(shell cd apps && ls -d */ | cut -d '/' -f 1)
VF2 ?=n
CV1811h ?=n
FEATURES :=
QEMU_ARGS :=
MEMORY_SIZE := 1024M
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
else ifeq ($(UNMATCHED),y)
FEATURES += hifive
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

ifeq ($(NET),y)
QEMU_ARGS += -device virtio-net-device,netdev=net0 \
			 -netdev user,id=net0,hostfwd=tcp::5555-:5555,hostfwd=udp::5555-:5555
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



install:
	@#cargo install --git  https://github.com/os-module/elfinfo
	@#cd $(TRACE_EXE_PATH) && cargo build --release

build:install compile


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

sdcard:fat32 testelf user

run:sdcard install compile
	@echo qemu booot $(SMP)
	$(call boot_qemu)
	@#rm ./kernel-qemu


fake_run:
	$(call boot_qemu)


board:install compile
	@rust-objcopy --strip-all $(KERNEL_FILE) -O binary $(OUTPUT)/testos.bin
	@cp $(OUTPUT)/testos.bin  /home/godones/projects/tftpboot/
	@cp $(OUTPUT)/testos.bin ./alien.bin

qemu:
	@rust-objcopy --strip-all $(OUTPUT)/boot -O binary $(OUTPUT)/testos.bin
	@cp $(OUTPUT)/testos.bin  /home/godones/projects/tftpboot/
	@cp $(OUTPUT)/testos.bin ./alien.bin

vf2:board
	@mkimage -f ./tools/vf2.its ./alien-vf2.itb
	@rm ./kernel-qemu
	@cp ./alien-vf2.itb /home/godones/projects/tftpboot/


cv1811h:board
	@mkimage -f ./tools/cv1811h.its ./alien-cv1811h.itb
	@rm ./kernel-qemu
	@cp ./alien-cv1811h.itb /home/godones/projects/tftpboot/

unmatched:board
	@mkimage -f ./tools/fu740.its ./alien-unmatched.itb
	@rm ./kernel-qemu
	@cp ./alien-unmatched.itb /home/godones/projects/tftpboot/

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
	if [ -d "sdcard" ]; then \
		sudo cp sdcard/* /fat -r; \
		sudo cp sdcard/* /fat/bin -r;\
	fi
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
	@dd if=/dev/zero of=$(IMG) bs=1M count=72
	@mkfs.fat -F 32 $(IMG)
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

gdb-server: sdcard install compile
	@qemu-system-riscv64 \
            -M virt\
            -bios $(BOOTLOADER) \
            -device loader,file=kernel-qemu,addr=$(KERNEL_ENTRY_PA) \
            -drive file=$(IMG),if=none,format=raw,id=x0 \
            -device virtio-blk-device,drive=x0 \
			-$(QEMU_ARGS) \
            -kernel  kernel-qemu\
            -smp 1 -m 1024M \
            -s -S

gdb-client:
	@riscv64-unknown-elf-gdb -ex 'file kernel-qemu' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'

kernel_asm:
	@riscv64-unknown-elf-objdump -d target/riscv64gc-unknown-none-elf/release/boot > kernel.asm
	@vim kernel.asm
	@rm kernel.asm

docs:
	cargo doc --open -p  kernel --target riscv64gc-unknown-none-elf --features $(FEATURES)
clean:
	@cargo clean
	@rm riscv.*
	@rm kernel-qemu
	@rm alien-*

.PHONY: all run clean fake_run

all:
	@mv alien.bin ./os.bin