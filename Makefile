TARGET := riscv64gc-unknown-none-elf
PROFILE := release
KERNEL := target/$(TARGET)/$(PROFILE)/kernel
NET ?=n
IMG := /tmp/fat32.img
SMP ?= 1
MEMORY_SIZE := 1024M

QEMU_ARGS += -nographic
ifeq ($(NET),y)
QEMU_ARGS += -device virtio-net-device,netdev=net0 \
			 -netdev user,id=net0,hostfwd=tcp::5555-:5555,hostfwd=udp::5555-:5555
endif


all:run

build:
	@echo "Building..."
	@cargo build --release -p kernel --target $(TARGET)

domains:
	make -C domains
	cp target/$(TARGET)/$(PROFILE)/gblk ./build/blk_domain.bin

run:domains build img
	qemu-system-riscv64 \
            -M virt \
            -bios default \
            -drive file=$(IMG),if=none,format=raw,id=x0 \
            -device virtio-blk-device,drive=x0 \
            -kernel $(KERNEL)\
            -$(QEMU_ARGS) \
            -smp $(SMP) -m $(MEMORY_SIZE) \
            -serial mon:stdio

img:
	@echo "Creating fat32.img..."
	if [ ! -f $(IMG) ]; then \
		dd if=/dev/zero of=$(IMG) bs=1M count=64; \
	fi
	@mkfs.fat -F 32 $(IMG)
	@-mkdir -p mnt
	@sudo mount $(IMG) mnt
	@sudo umount mnt
	@rm -rf mnt

gdb-server: domains build img
	@qemu-system-riscv64 \
            -M virt\
            -bios default \
            -drive file=$(IMG),if=none,format=raw,id=x0 \
            -device virtio-blk-device,drive=x0 \
            -kernel $(KERNEL)\
			-$(QEMU_ARGS) \
			-smp $(SMP) -m $(MEMORY_SIZE) \
            -s -S

gdb-client:
	@riscv64-unknown-elf-gdb -ex 'file $(KERNEL)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'



.PHONY:build domains