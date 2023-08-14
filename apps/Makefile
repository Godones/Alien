APPS := $(shell ls -d */)
compile := cargo build --release
mode := release
target := ../target/riscv64gc-unknown-none-elf/$(mode)/
APPS_NAME := $(shell ls -d */ | cut -d '/' -f 1)



all:build
	@echo "Moving apps to /fat32/bin"
	@$(foreach dir, $(BUILD_CRATES), (sudo cp $(target)$(dir) /fat/bin/$(dir););)
	@sync


build:
	@echo "Building apps"
	@$(foreach dir, $(BUILD_CRATES), (cd $(dir) && $(compile) && cd ..);)


BUILD_CRATES := \
	shell \
	init \
	mkdir \
	ls \
	cat \
	run_test \
	tests \
	sleep \
	socket_test \
	print
#	slint \
#	memory-game \
#	guitest