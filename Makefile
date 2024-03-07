TARGET := riscv64gc-unknown-none-elf
PROFILE := release
KERNEL := target/$(TARGET)/$(PROFILE)/kernel
NET ?=n
IMG := /tmp/fat32.img
SMP ?= 1
MEMORY_SIZE := 1024M
LOG ?=INFO

QEMU_ARGS += -nographic
ifeq ($(NET),y)
QEMU_ARGS += -device virtio-net-device,netdev=net0 \
			 -netdev user,id=net0,hostfwd=tcp::5555-:5555,hostfwd=udp::5555-:5555
endif




domains += 	gblk gfatfs gcache_blk ggoldfish gvfs gshadow_blk gextern-interrupt

all:run

build:
	@echo "Building..."
	@ LOG=$(LOG) cargo build --release -p kernel --target $(TARGET)

domains:
	make -C domains all  DOMAIN_LIST="$(domains)" LOG=$(LOG)
	$(foreach dir, $(domains), cp target/$(TARGET)/$(PROFILE)/$(dir) ./build/$(dir)_domain.bin;)

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
	@-sudo mount $(IMG) mnt
	@sudo touch mnt/empty
	@sudo touch mnt/f1.txt
	@echo "Hello, world!" | sudo tee mnt/f1.txt
	@sudo umount mnt
	@sudo rm -rf mnt

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



.PHONY:build domains gdb-client gdb-server img