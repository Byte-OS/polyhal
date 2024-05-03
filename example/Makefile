# Building
ARCH := riscv64
ifeq ($(ARCH), x86_64)
  TARGET := x86_64-unknown-none
  QEMU_EXEC += qemu-system-x86_64 \
				-machine q35 \
				-kernel $(KERNEL_ELF) \
				-cpu IvyBridge-v2
  BUS := pci
else ifeq ($(ARCH), riscv64)
  TARGET := riscv64gc-unknown-none-elf
  QEMU_EXEC += qemu-system-$(ARCH) \
				-machine virt \
				-kernel $(KERNEL_BIN)
else ifeq ($(ARCH), aarch64)
  TARGET := aarch64-unknown-none-softfloat
  QEMU_EXEC += qemu-system-$(ARCH) \
				-cpu cortex-a72 \
				-machine virt \
				-kernel $(KERNEL_BIN)
else ifeq ($(ARCH), loongarch64)
  TARGET := loongarch64-unknown-none
  QEMU_EXEC += qemu-system-$(ARCH) -kernel $(KERNEL_ELF)
  BUILD_ARGS += -Z build-std=core,alloc
  BUS := pci
else
  $(error "ARCH" must be one of "x86_64", "riscv64", "aarch64" or "loongarch64")
endif

KERNEL_ELF := target/$(TARGET)/release/example
KERNEL_BIN := $(KERNEL_ELF).bin

# Binutils
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64

build: env $(KERNEL_BIN) 

env:
	(rustup target list | grep "$(TARGET) (installed)") || rustup target add $(TARGET)
	cargo install cargo-binutils
	rustup component add rust-src
	rustup component add llvm-tools-preview

$(KERNEL_BIN): kernel
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

kernel:
	@echo Platform: $(BOARD)
	@cargo build $(BUILD_ARGS) --release --target $(TARGET)

clean:
	@cargo clean

run: run-inner

QEMU_EXEC += -nographic -smp 1

run-inner: build
	$(QEMU_EXEC)

test:
	make ARCH=aarch64 run
	make ARCH=riscv64 run
	make ARCH=x86_64 run
#	make ARCH=loongarch64 run

.PHONY: build env kernel clean run-inner
