.DEFAULT_GOAL := help

CARGO ?= cargo
RUSTUP ?= rustup
WINDOWS_LINKER ?= x86_64-w64-mingw32-gcc
LINUX_LINKER ?= x86_64-unknown-linux-gnu-gcc
SLINT_ARGS := --locked --release --features slint-ui --bin rusty-pomodoro-slint

.PHONY: help dev fmt fmt-check build build-macos build-windows build-linux package-macos

help:
	@printf '%s\n' \
		'make dev          Run Slint app for local development' \
		'make fmt          Format Rust source with rustfmt' \
		'make fmt-check    Check Rust formatting' \
		'make build        Build release binaries for macOS, Windows, and Linux' \
		'make build-macos  Build macOS arm64 and x86_64 binaries' \
		'make build-windows Build Windows x86_64 binary (MinGW cross-linker required)' \
		'make build-linux  Build Linux x86_64 binary (cross-linker required)' \
		'make package-macos Package native AppKit app bundle'

dev:
	$(CARGO) run --locked --features slint-ui --bin rusty-pomodoro-slint

fmt:
	$(CARGO) fmt --all

fmt-check:
	$(CARGO) fmt --all -- --check

build: build-macos build-windows build-linux

build-macos:
	$(RUSTUP) target add aarch64-apple-darwin x86_64-apple-darwin
	$(CARGO) build $(SLINT_ARGS) --target aarch64-apple-darwin
	$(CARGO) build $(SLINT_ARGS) --target x86_64-apple-darwin

build-windows:
	$(RUSTUP) target add x86_64-pc-windows-gnu
	CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER="$(WINDOWS_LINKER)" $(CARGO) build $(SLINT_ARGS) --target x86_64-pc-windows-gnu

build-linux:
	$(RUSTUP) target add x86_64-unknown-linux-gnu
	CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="$(LINUX_LINKER)" $(CARGO) build $(SLINT_ARGS) --target x86_64-unknown-linux-gnu

package-macos:
	./scripts/package-macos.sh
