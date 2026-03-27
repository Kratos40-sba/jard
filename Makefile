.PHONY: build test run lint clean docker-build help

# Default goal
help:
	@echo "Jard Build System"
	@echo "-----------------"
	@echo "build        - Build the project in release mode"
	@echo "test         - Run all tests"
	@echo "run          - Run the project locally"
	@echo "lint         - Run clippy and check formatting"
	@echo "clean        - Clean the build artifacts"
	@echo "docker-build - Build the production Docker image"

build:
	cargo build --release

test:
	cargo test

run:
	cargo run

lint:
	cargo clippy --all-targets --all-features -- -D warnings
	cargo fmt --check

clean:
	cargo clean

docker-build:
	docker build -t jard:latest .
