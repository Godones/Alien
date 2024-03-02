TARGET := riscv64gc-unknown-none-elf
PROFILE := release
LOG ?=INFO
DOMAIN_LIST ?=


all:
	@echo build: $(DOMAIN_LIST)
	$(foreach dir, $(DOMAIN_LIST), (cd generated/$(dir) && LOG=$(LOG) cargo build --release -p $(dir));)




.PHONY: $(DOMAIN_LIST) all