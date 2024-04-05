TARGET := riscv64gc-unknown-none-elf
PROFILE := release
LOG ?=INFO
DOMAIN_LIST ?=


all:
	@echo build: $(DOMAIN_LIST)
	@$(foreach dir, $(DOMAIN_LIST), (cd generated/$(dir) && LOG=$(LOG) cargo build --release -p $(dir));)


fix:
	@$(foreach dir, $(DOMAIN_LIST), (cd generated/$(dir) &&  cargo fix -p $(dir) --target $(TARGET) --allow-dirty --allow-staged);)

.PHONY: all fix