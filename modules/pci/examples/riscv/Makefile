arch ?= riscv64
board ?= qemu
target := $(arch)imac-unknown-none-elf
mode := release
kernel := target/$(target)/$(mode)/riscv
bin := target/$(target)/$(mode)/riscv.bin
img := target/$(target)/$(mode)/nvme.img

sysroot := $(shell rustc --print sysroot)
objdump := $(shell find $(sysroot) -name llvm-objdump) --arch-name=$(arch)
objcopy := $(shell find $(sysroot) -name llvm-objcopy)

BUILD_ARGS += --target $(target)
ifeq ($(mode), release)
	BUILD_ARGS += --release
endif
ifeq ($(board), fu740)
	BUILD_ARGS += --features board-fu740
endif

.PHONY: kernel build clean qemu run

build: kernel $(bin)

kernel:
	cargo build $(BUILD_ARGS)

$(bin): kernel
	$(objcopy) $(kernel) --strip-all -O binary $(bin)

asm:
	$(objdump) -d $(kernel) | less

sym:
	$(objdump) -t $(kernel) | less

header:
	$(objdump) -x $(kernel) | less

clean:
	cargo clean

qemu: $(img)
	qemu-system-$(arch) \
		-machine virt \
		-serial mon:stdio \
		-display none \
		-bios default \
		-kernel $(kernel) \
		-device ahci,id=ahci0 \
		-drive file=$(img),if=none,format=raw,id=nvme0 \
		-device nvme,serial=deadbeef,drive=nvme0 \
		-netdev user,id=net0 \
		-device e1000e,netdev=net0

$(img):
	dd if=/dev/zero of=$@ bs=512 count=32

run: build qemu
