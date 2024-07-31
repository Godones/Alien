TARGET = riscv64gc-unknown-linux-musl
compile := cargo build --release -Z build-std=core,std,panic_abort
mode := release
target := ../../target/$(TARGET)/$(mode)/
FSMOUNT := ../../diskfs



all:build
	@echo "Moving apps to ../diskfs/bin"
	@$(foreach dir, $(BUILD_CRATES), (sudo cp $(target)$(dir) $(FSMOUNT)/$(dir););)


build:
	@echo "Building apps"
	$(foreach dir, $(BUILD_CRATES), ($(compile) -p $(dir));)


BUILD_CRATES := \
	hello \
	async_test