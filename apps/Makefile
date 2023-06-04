APPS := $(shell ls -d */)
compile := cargo build --release
mode := release
target := ../target/riscv64gc-unknown-none-elf/$(mode)/
APPS_NAME := $(shell ls -d */ | cut -d '/' -f 1)



all:build
	@# move to /fat32
	@echo "Moving apps to /fat32"
	@$(foreach dir, $(BUILD_CRATES), (sudo cp $(target)$(dir) /fat/$(dir););)
	@sync



build:
	@echo "Building apps"
	@$(foreach dir, $(BUILD_CRATES), (cd $(dir) && $(compile) && cd ..);)
	@#$(foreach dir, $(BUILD_CRATES), echo $(dir);)

fmt:
	@echo "Formatting code"
	@$(foreach dir, $(APPS), (cd $(dir) && cargo fmt && cd ..);)

clean:
	@echo "Cleaning apps"
	@$(foreach dir, $(APPS), (cd $(dir) && cargo clean && cd ..);)
	@$(foreach dir, $(APPS), (cd $(dir) && rm -rf target && cd ..);)



clock:
	@$(foreach dir, $(APPS), (cd $(dir) && rm -rf Cargo.lock && cd ..);)














BUILD_CRATES := \
	shell \
	init \
	mkdir \
	ls \
	cat \
	mmmap \
	run_test