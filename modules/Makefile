APPS := $(shell ls -d */)
fmt:
	@echo "Formatting code"
	@$(foreach dir, $(APPS), (cd $(dir) && cargo fmt && cd ..);)

clean:
	@echo "Cleaning apps"
	@$(foreach dir, $(APPS), (cd $(dir) && cargo clean && cd ..);)
	@$(foreach dir, $(APPS), (cd $(dir) && rm -rf target && cd ..);)



clock:
	@$(foreach dir, $(APPS), (cd $(dir) && rm -rf Cargo.lock && cd ..);)