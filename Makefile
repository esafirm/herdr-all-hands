PLUGIN_ID := $(shell awk -F'"' '/^id = "/ { print $$2; exit }' herdr-plugin.toml)
BIN := bin/herdr-pluck

.DEFAULT_GOAL := help
.PHONY: help build install reinstall uninstall status check

help: ## Show available targets
	@awk 'BEGIN { FS = ":.*## " } /^[a-z-]+:.*## / { printf "  %-10s %s\n", $$1, $$2 }' $(MAKEFILE_LIST)

build: ## Build the release binary from source into ./bin
	cargo build --release
	mkdir -p bin
	cp target/release/herdr-pluck $(BIN)
	chmod +x $(BIN)

# `herdr plugin link` does not run the manifest build step, so build first.
# The link points at this checkout, so later `make build` runs take effect directly.
install: build ## Build from source and link this checkout into Herdr
	herdr plugin link .
	herdr plugin action list --plugin $(PLUGIN_ID)

reinstall: uninstall install ## Unlink and link again

uninstall: ## Unlink this plugin from Herdr
	-herdr plugin unlink $(PLUGIN_ID)

status: ## Show the installed plugin and its config directory
	herdr plugin list
	@echo "config: $$(herdr plugin config-dir $(PLUGIN_ID))"

check: ## Run fmt, tests and clippy
	cargo fmt --all -- --check
	cargo test --all-features
	cargo clippy --all-targets --all
