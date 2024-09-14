# Makefile to test the polyhal with multiple architectures

all:

test-build:
	cargo build --release --all-features --target riscv64gc-unknown-none-elf
	cargo build --release --all-features --target aarch64-unknown-none-softfloat
	cargo build --release --all-features --target x86_64-unknown-none
	cargo build --release --all-features --target loongarch64-unknown-none

test-clippy:
	cargo clippy --all-features --target riscv64gc-unknown-none-elf
	cargo clippy --all-features --target aarch64-unknown-none-softfloat
	cargo clippy --all-features --target x86_64-unknown-none
	cargo clippy --all-features --target loongarch64-unknown-none


.PHONY: test
