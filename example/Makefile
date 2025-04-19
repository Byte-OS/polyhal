# Building
ARCH := riscv64
TEST := false
GUI  := false
SMP  := 1
FEATURES := 
BOOT_ARGS := 
GRUB_MKRESCUE := $(shell which grub-mkrescue || which grub2-mkrescue)

export RUSTFLAGS := 
QEMU_EXEC += qemu-system-$(ARCH)

ifeq ($(ARCH), x86_64)
  RUSTFLAGS += -Clink-arg=-no-pie
  TARGET := x86_64-unknown-none
  QEMU_EXEC += -machine q35 \
				-kernel $(KERNEL_ELF) \
				-cpu IvyBridge-v2
  BUS := pci
else ifeq ($(ARCH), riscv64)
  TARGET := riscv64gc-unknown-none-elf
  QEMU_EXEC += -machine virt -kernel $(KERNEL_BIN)
else ifeq ($(ARCH), aarch64)
  TARGET := aarch64-unknown-none-softfloat
  QEMU_EXEC += -cpu cortex-a72 -machine virt -kernel $(KERNEL_BIN)
else ifeq ($(ARCH), loongarch64)
  TARGET := loongarch64-unknown-none
  QEMU_EXEC += -kernel $(KERNEL_ELF) -M virt -m 1G
  BUS := pci
else
  $(error "ARCH" must be one of "x86_64", "riscv64", "aarch64" or "loongarch64")
endif
KERNEL_ELF := ../target/$(TARGET)/release/example
KERNEL_BIN := $(KERNEL_ELF).bin

# Binutils
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64

ifneq ($(GUI), true)
QEMU_EXEC += -nographic
else
FEATURES += polyhal-boot/graphic polyhal/graphic
QEMU_EXEC += -serial stdio -vga std
endif 
QEMU_EXEC += -smp $(SMP)
ifneq ($(SMP), 1)
QEMU_EXEC += -D ../qemu-%d.log -d in_asm,int,pcall,cpu_reset,tid,guest_errors
else
QEMU_EXEC += -D ../qemu.log -d in_asm,int,pcall,cpu_reset,guest_errors
endif
QEMU_EXEC += -append "$(BOOT_ARGS)"

build: $(KERNEL_BIN) 

$(KERNEL_BIN): kernel
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

kernel:
	@echo Platform: $(BOARD)
	@cargo build $(BUILD_ARGS) --release --features "$(FEATURES)" --target $(TARGET)

clean:
	@cargo clean

run: run-inner

run-inner: build
	rm -f ../qemu-*.log
	$(QEMU_EXEC)

debug: build
	$(QEMU_EXEC) -s -S

test:
	make ARCH=aarch64 run
	make ARCH=riscv64 run
	make ARCH=x86_64 run
	make ARCH=loongarch64 run

iso: build
	cp $(KERNEL_ELF) iso/example
	$(GRUB_MKRESCUE) -o bootable.iso iso

boot-iso: iso
	qemu-system-x86_64 -cdrom bootable.iso -serial stdio -vga vmware

gdb:
	gdb \
	-ex 'file $(KERNEL_ELF)' \
	-ex 'set arch x86_64' \
	-ex 'target remote localhost:1234'

.PHONY: build env kernel clean clean-log run-inner
