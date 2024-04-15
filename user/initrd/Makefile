VERSION = 1.33.1
BB = busybox
COMPILER = riscv64-linux-musl-
ARCH = riscv64
initrd = initramfs


all:
	@make download
	@if [ -f initramfs.cpio.gz ]; then \
		echo "initramfs.cpio.gz exists"; \
	else \
	  	make config; \
		make build; \
		make initramfs; \
	fi


download:
	@if [ -d $(BB) ]; then \
    		echo "Busybox exists"; \
    	else \
    		echo "Busybox does not exist"; \
    		wget http://busybox.net/downloads/busybox-$(VERSION).tar.bz2; \
    		tar -xvf busybox-$(VERSION).tar.bz2 && mv busybox-$(VERSION) $(BB); \
	fi

config:
	sudo apt install libncurses5-dev libncursesw5-dev
	CROSS_COMPILE=$(COMPILER) ARCH=$(ARCH) make defconfig -C $(BB)
	CROSS_COMPILE=$(COMPILER) ARCH=$(ARCH) make menuconfig -C $(BB)

build:
	CROSS_COMPILE=$(COMPILER) ARCH=$(ARCH) make -j $(nproc) -C $(BB)
	CROSS_COMPILE=$(COMPILER) ARCH=$(ARCH) make install -C $(BB)

initramfs:
	@if [ -d $(initrd) ]; then \
		echo "initramfs exists"; \
		rm -rf $(initrd)/*; \
	else \
		echo "initramfs does not exist"; \
		mkdir $(initrd); \
	fi
	cp -r $(BB)/_install/* $(initrd)
	-#mkdir -p $(initrd)/dev
	-#mkdir -p $(initrd)/proc
	-#mkdir -p $(initrd)/sys
	-#mkdir -p $(initrd)/tmp
	@if [ -f init ]; then \
		echo "init exists"; \
	else \
		echo "init does not exist"; \
		echo "#!/bin/sh\nmount -t proc none /proc\nmount -t sysfs none /sys\nmount -t devtmpfs none /dev\nexec /bin/sh" > $(initrd)/init; \
		chmod +x $(initrd)/init; \
	fi
	cd $(initrd) && find . -print0 | cpio --null -ov --format=newc | gzip -9 > ../initramfs.cpio.gz


clean:
	make clean -C $(BB)
	rm initramfs.cpio.gz

.PHONY: clean initramfs