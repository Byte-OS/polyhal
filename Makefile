# Makefile to test the polyhal with multiple architectures

all:

.PHONY: example
example:
	make -C example run

clean:
	rm -rf target/

test-build:
	cargo build --all-features --target riscv64gc-unknown-none-elf
	cargo build --all-features --target aarch64-unknown-none-softfloat
	cargo build --all-features --target x86_64-unknown-none
	cargo build --all-features --target loongarch64-unknown-none

test-clippy:
	cargo clippy --all-features --target riscv64gc-unknown-none-elf
	cargo clippy --all-features --target aarch64-unknown-none-softfloat
	cargo clippy --all-features --target x86_64-unknown-none
	cargo clippy --all-features --target loongarch64-unknown-none
