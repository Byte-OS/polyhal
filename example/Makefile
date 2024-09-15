# Building
ARCH := riscv64
TEST := false
GUI  := false
SMP  := 1

ifeq ($(TEST), true)
QEMU_EXEC := timeout 40
endif

ifeq ($(ARCH), x86_64)
  TARGET := x86_64-unknown-none
  KERNEL_ELF := target/$(TARGET)/release/example
  KERNEL_BIN := $(KERNEL_ELF).bin
  QEMU_EXEC += qemu-system-x86_64 \
				-machine q35 \
				-kernel $(KERNEL_ELF) \
				-cpu IvyBridge-v2
  BUS := pci
else ifeq ($(ARCH), riscv64)
  TARGET := riscv64gc-unknown-none-elf
  KERNEL_ELF := target/$(TARGET)/release/example
  KERNEL_BIN := $(KERNEL_ELF).bin
  QEMU_EXEC += qemu-system-$(ARCH) \
				-machine virt \
				-kernel $(KERNEL_BIN)
else ifeq ($(ARCH), aarch64)
  TARGET := aarch64-unknown-none-softfloat
  KERNEL_ELF := target/$(TARGET)/release/example
  KERNEL_BIN := $(KERNEL_ELF).bin  
  QEMU_EXEC += qemu-system-$(ARCH) \
				-cpu cortex-a72 \
				-machine virt \
				-kernel $(KERNEL_BIN)
else ifeq ($(ARCH), loongarch64)
  TARGET := loongarch64-unknown-none
  KERNEL_ELF := target/$(TARGET)/release/example
  KERNEL_BIN := $(KERNEL_ELF).bin
  QEMU_EXEC += qemu-system-$(ARCH) -kernel $(KERNEL_ELF) -M virt -m 1G
  BUILD_ARGS += -Z build-std=core,alloc
  BUS := pci
else
  $(error "ARCH" must be one of "x86_64", "riscv64", "aarch64" or "loongarch64")
endif

# Binutils
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64

ifneq ($(GUI), true)
QEMU_EXEC += -nographic
else
QEMU_EXEC += -serial stdio -vga std
endif 
QEMU_EXEC += -smp $(SMP)
QEMU_EXEC += -D qemu.log -d in_asm,int,pcall,cpu_reset,guest_errors
ifeq ($(TEST), true)
QEMU_EXEC += >output.log 2>&1 || echo "QEMU exited"
endif

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

clean-log:
	@rm qemu*.log

clean:
	@cargo clean

run: run-inner

run-inner: build
	$(QEMU_EXEC)
ifeq ($(TEST), true)
	grep "\[kernel\] Hello, world!" output.log
endif

debug: build
	$(QEMU_EXEC) -s -S

test:
	make ARCH=aarch64 run
	make ARCH=riscv64 run
	make ARCH=x86_64 run
	make ARCH=loongarch64 run

iso: build
	cp $(KERNEL_ELF) iso/example
	grub2-mkrescue -o bootable.iso iso

boot-iso: iso
	qemu-system-x86_64 -cdrom bootable.iso -serial stdio

gdb:
	gdb \
	-ex 'file $(KERNEL_ELF)' \
	-ex 'set arch x86_64' \
	-ex 'target remote localhost:1234'

.PHONY: build env kernel clean clean-log run-inner
