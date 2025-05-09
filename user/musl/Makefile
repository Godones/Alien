TARGET = riscv64gc-unknown-linux-musl
compile := cargo build --release -Z build-std=core,std,panic_abort --target=$(TARGET)
mode := release
target := ../../target/$(TARGET)/$(mode)/
FSMOUNT := ../../diskfs



all:build
	@echo "Moving apps to ../diskfs/bin"
	$(foreach dir, $(BUILD_CRATES), (sudo cp $(target)$(dir) $(FSMOUNT)/$(dir););)
	@sudo cp $(target)syscall_ebpf $(FSMOUNT)/syscall_ebpf


build:
	@echo "Building apps"
	$(foreach dir, $(BUILD_CRATES), ($(compile) -p $(dir));)
	cd syscall_ebpf && $(compile) --target-dir ../../../target


BUILD_CRATES := \
	hello \
	async_test \
	ftest