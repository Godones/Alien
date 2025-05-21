ifeq ($(ARCH), x86_64)
	CROSS_COMPILE=x86_64-linux-musl-
else ifeq ($(ARCH), riscv64)
	CROSS_COMPILE=riscv64-linux-musl-
endif


FSMOUNT := ../../diskfs
CC=$(CROSS_COMPILE)gcc


all:build
	@$(foreach file, $(BUILD_LIST), (sudo cp $(file) $(FSMOUNT)/$(dir););)

build:
	@echo "Building C apps"
	$(foreach file, $(BUILD_LIST), $(CC) -static -o $(file) $(file).c;)

clean:
	$(foreach file, $(BUILD_LIST), rm $(file);)

BUILD_LIST := \
	eventfd_test \
	test_epoll \


.PHONY: install clean all