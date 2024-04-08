# arch crate

> A crate help you to write a os that support multiple platforms.

English | [中文简体](./README.CN.md)

## Supported platforms

| Platform | Board |
| ---      |  ---  |
| riscv64  | qemu |
| x86_64   | qemu |
| aarch64  | qemu |
| loongarch64 | qemu |


## Example

Here is an simple example.

## Used OS

### ByteOS

How to run?

##### Prepare

```shell
git clone https://github.com/Byte-OS/ByteOS.git
cd ByteOS
git reset 655eef3e38b5a85baaab4b2ba33832fbb299f19a --hard
git clone https://github.com/Byte-OS/arch.git
```

##### Run on a specific platform

run on riscv64

```shell
make ARCH=riscv64 LOG=error run
```

Change ARCH value if you want to run on another platform.

