
.PHONY: test
test: ## Run the tests with nextest
	cargo nextest run


.PHONY: test-cargo
test-cargo: ## Run the tests with built-in cargo test
	cargo test --all-features

.PHONY: doctest
doctest: ## Run doc tests 
	cargo test --doc --all-features

.PHONY: test-all
test-all: test doctest ## Run all tests including doctests

.PHONY: check
check: ## Run Cargo check
	cargo check --all-targets --all-features

.PHONY: fmt
fmt: ## Format code
	cargo fmt --all

.PHONY: fmt-check
fmt-check: ## Check code formatting
	cargo fmt --all -- --check

.PHONY: clippy
clippy: ## Run clippy
	cargo clippy --all-targets --all-features -- -D warnings

.PHONY: doc
doc: ## Build documentation
	RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features

.PHONY: ci
ci: fmt-check clippy test-cargo ## Run all CI checks

.PHONY: clippy-fix
clippy-fix: ## Run clippy with automatic fixes
	cargo clippy --all-targets --all-features --fix

.PHONY: install-nextest
install-nextest: ## Install cargo-nextest
	@echo "Installing cargo-nextest..."
	cargo install cargo-nextest --locked
	@echo "cargo-nextest installed"

.DEFAULT_GOAL := help
.PHONY: help
help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'
