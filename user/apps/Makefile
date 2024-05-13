APPS := $(shell ls -d */)
compile := cargo build --release
mode := release
target := ../../target/riscv64gc-unknown-none-elf/$(mode)/
APPS_NAME := $(shell ls -d */ | cut -d '/' -f 1)
GUI ?=n
FSMOUNT := ../../diskfs

all:build
	@echo "Moving apps to ../diskfs/bin"
	@$(foreach dir, $(BUILD_CRATES), (sudo cp $(target)$(dir) $(FSMOUNT)/$(dir););)


build:
	@echo "Building apps"
	@$(foreach dir, $(BUILD_CRATES), (cd $(dir) && $(compile) && cd ..);)


BUILD_CRATES := \
	init \
	dtest \
	domain \
	dgpu \
	dtask \
	dlog \
	ping \
	client \
#	run_test \
	tests \
	sleep \
	socket_test \
	final_test \
	ls \
	cat \
	mkdir \
	print \

#ifeq ($(GUI),y)
#	BUILD_CRATES += egui \
#					sysinfo \
#					todo \
#					slint \
#					memory-game \
#					printdemo
#endif