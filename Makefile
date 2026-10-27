.PHONY: check esp32-check esp32-build espidf-check espidf-build fmt fmt-check lint help
.DEFAULT_GOAL := help

check: esp32-check

esp32-check:
	cd no_std && cargo check --examples --locked

esp32-build:
	cd no_std && cargo build --examples --locked

espidf-check:
	cd std && cargo check --locked

espidf-build:
	cd std && cargo build --locked

fmt:
	cd no_std && cargo fmt --all
	cd std && cargo fmt --all

fmt-check:
	cd no_std && cargo fmt --all -- --check
	cd std && cargo fmt --all -- --check

lint:
	cd no_std && cargo clippy --examples --locked -- -D warnings

help:
	@echo "먼저 . ~/export-esp.sh"
	@echo "make esp32-check / esp32-build / espidf-check / espidf-build"
	@echo "make fmt / fmt-check / lint"
