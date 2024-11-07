#!/bin/bash

echo "ARGS1 $1"

rust-objcopy --binary-architecture=riscv64 $1 --strip-all -O binary $1.bin

qemu-system-riscv64 \
    -machine virt \
    -kernel $1.bin \
    -nographic -smp 1 \
    -D qemu.log -d in_asm,int,pcall,cpu_reset,guest_errors
