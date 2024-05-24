TARGET := riscv64gc-unknown-none-elf
PROFILE := release
KERNEL := target/$(TARGET)/$(PROFILE)/kernel
NET ?=y
SMP ?=1
MEMORY_SIZE := 1024M
LOG ?=
GUI ?=n
FS ?=fat
IMG := build/sdcard.img
FSMOUNT := ./diskfs
FEATURES := smp
name ?=


comma:= ,
empty:=
space:= $(empty) $(empty)

QEMU_ARGS :=

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

QEMU_ARGS += -initrd ./build/initramfs.cpio.gz
QEMU_ARGS += -append "rdinit=/init"

FEATURES := $(subst $(space),$(comma),$(FEATURES))
all:run

build:
	@echo "Building..."
	@ LOG=$(LOG) cargo build --release -p kernel --target $(TARGET) --features $(FEATURES)

run: domains sdcard initrd build
	qemu-system-riscv64 \
            -M virt \
            -bios default \
            -drive file=$(IMG),if=none,format=raw,id=x0 \
            -device virtio-blk-device,drive=x0 \
            -kernel $(KERNEL)\
            -$(QEMU_ARGS) \
            -smp $(SMP) -m $(MEMORY_SIZE) \
            -serial mon:stdio
	-#rm $(IMG)

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


sdcard:$(FS) mount user #domains
	@sudo cp build/disk/* $(FSMOUNT)/
	@-sudo cp user/bin/* $(FSMOUNT)/
	@sudo mkdir -p $(FSMOUNT)/domains
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
	mkdir $(FSMOUNT)
	@sudo mount $(IMG) $(FSMOUNT)
	@sudo rm -rf $(FSMOUNT)/*


domains:
	if [ ! -d "build/disk" ]; then mkdir -p build/disk; fi
	if [ ! -d "build/init" ]; then mkdir -p build/init; fi
	cd domains && cargo domain build-all -l "$(LOG)"
	cp domains/build/disk/* build/disk/ -r
	cp domains/build/init/* build/init/ -r

domain:
	cd domains && cargo domain build -n $(name) -l "$(LOG)"
	cp domains/build/disk/* build/disk/ -r
	cp domains/build/init/* build/init/ -r
	@make initrd

initrd:
	@make -C user/initrd
	@mkdir -p ./initrd
	@cp ./build/init/g* ./initrd
	@cp ./user/initrd/initramfs/* ./initrd -r
	@-cp ./user/bin/* ./initrd/bin -r
	@cd ./initrd && find . -print0 | cpio --null -ov --format=newc | gzip -9 > ../build/initramfs.cpio.gz && cd ..
	@rm -rf ./initrd


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
	rm build/g*
	cargo clean

kernel_asm:
	@riscv64-unknown-elf-objdump -d target/riscv64gc-unknown-none-elf/release/kernel > kernel.asm
	@vim kernel.asm
	@rm kernel.asm


.PHONY:build domains gdb-client gdb-server img sdcard user mount $(FS) fix initrd