TARGET := riscv64gc-unknown-none-elf
PROFILE := release
KERNEL := target/$(TARGET)/$(PROFILE)/kernel
NET ?=n
SMP ?= 1
MEMORY_SIZE := 1024M
LOG ?=INFO
GUI ?=n
FS ?=fat
IMG := build/sdcard.img
FSMOUNT := ./diskfs
FEATURES := smp


QEMU_ARGS += -nographic

ifeq ($(GUI),y)
	QEMU_ARGS += -device virtio-gpu-device
	FEATURES += gui
else
	QEMU_ARGS += -nographic
endif

ifeq ($(NET),y)
QEMU_ARGS += -device virtio-net-device,netdev=net0 \
			 -netdev user,id=net0,hostfwd=tcp::5555-:5555,hostfwd=udp::5555-:5555
endif



domains += 	gblk gcache_blk ggoldfish gvfs gshadow_blk gextern-interrupt gdevices ggpu guart gtask \
		gsyscall gbuf_uart gvirtio-mmio-net ginput


all:run

build:
	@echo "Building..."
	@ LOG=$(LOG) cargo build --release -p kernel --target $(TARGET) --features $(FEATURES)

run: sdcard domains build
	qemu-system-riscv64 \
            -M virt \
            -bios default \
            -drive file=$(IMG),if=none,format=raw,id=x0 \
            -device virtio-blk-device,drive=x0 \
            -kernel $(KERNEL)\
            -$(QEMU_ARGS) \
            -smp $(SMP) -m $(MEMORY_SIZE) \
            -serial mon:stdio
	-rm $(IMG)

fake_run:
	qemu-system-riscv64 \
			-M virt \
			-bios default \
			-drive file=$(IMG),if=none,format=raw,id=x0 \
			-device virtio-blk-device,drive=x0 \
			-kernel $(KERNEL)\
			-$(QEMU_ARGS) \
			-smp $(SMP) -m $(MEMORY_SIZE) \
			-serial mon:stdio

user:
	@echo "Building user apps"
	@make all -C ./user/apps
	@echo "Building user apps done"

sdcard:$(FS) mount user
	@sudo ls $(FSMOUNT)
	@sudo umount $(FSMOUNT)
	@rm -rf $(FSMOUNT)

fat:
	dd if=/dev/zero of=$(IMG) bs=1M count=72;
	@mkfs.fat -F 32 $(IMG)


mount:
	@echo "Mounting $(IMG) to $(FSMOUNT)"
	@-sudo umount $(FSMOUNT);
	@sudo rm -rf $(FSMOUNT)
	@-mkdir $(FSMOUNT)
	@sudo mount $(IMG) $(FSMOUNT)
	@sudo rm -rf $(FSMOUNT)/*



domains:
	make -C domains all  DOMAIN_LIST="$(domains)" LOG=$(LOG)
	$(foreach dir, $(domains), cp target/$(TARGET)/$(PROFILE)/$(dir) ./build/$(dir)_domain.bin;)


gdb-server: domains build sdcard
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

clean:
	rm -rf target/
	rm build/*.bin

.PHONY:build domains gdb-client gdb-server img sdcard user mount $(FS)