TARGET := riscv64gc-unknown-none-elf
PROFILE := release
LOG ?=INFO

all:
	$(foreach dir, $(domains), (cd generated/$(dir) && LOG=$(LOG) cargo build --release -p $(dir) && cd ../..);)


domains = 	gblk \
			gfatfs \

.PHONY: $(domains)