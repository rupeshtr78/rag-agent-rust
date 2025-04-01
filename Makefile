# Project variables
PROJECT_NAME := rag-agent-rust
CARGO := cargo

# install cargo if not installed
ifndef CARGO
$(shell if ! command -v cargo &> /dev/null; then \
	echo "Cargo not found. Installing..."; \
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh; \
	echo "Please restart your terminal or run 'source $$(rustup completions bash)'"; \
	exit 1; \
fi)
endif

# Default target
.DEFAULT_GOAL := help

.PHONY: help
help: ## Display this help message
	@echo "Usage: make [target]"
	@echo ""
	@echo "Available targets:"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-15s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

.PHONY: build
build: ## Build the project in debug mode
	$(CARGO) build

.PHONY: release
build-release: ## Build the project in release mode
	$(CARGO) build --release

.PHONY: run
run: ## Run the application with default arguments
	$(CARGO) run -- --help

.PHONY: test
test: ## Run tests
	$(CARGO) test

.PHONY: clippy
clippy: ## Run clippy linter
	$(CARGO) clippy -- -D warnings

.PHONY: fmt
fmt: ## Format the code
	$(CARGO) fmt

.PHONY: clean
clean: ## Clean the project
	$(CARGO) clean

.PHONY: load-sample
load-sample: ## Load sample files into the database (example usage)
	$(CARGO) run -- load -p sample/

.PHONY: query-sample
query-sample: ## Query the sample database (example usage)
	$(CARGO) run -- rag-query -t sample_table -d sample_db -i "what is temperature"

.PHONY: chat-sample
chat-sample: ## Start an interactive chat session (example usage)
	$(CARGO) run -- chat -p "what is mirostat"