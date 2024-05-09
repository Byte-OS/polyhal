# arch crate

> A crate help you to write a os that support multiple platforms.

English | [中文简体](./README.CN.md)

[Wiki](https://github.com/Byte-OS/polyhal/wiki)

## Supported platforms

| Platform | Board |
| ---      |  ---  |
| riscv64  | qemu |
| x86_64   | qemu |
| aarch64  | qemu |
| loongarch64 | qemu |


## Example

Here is an simple example in the example dir.

### Run

``` shell
cd example
# Run on riscv64
make ARCH=riscv64 run
# Run on x86_64
make ARCH=x86_64 run
# Run on aarch64
make ARCH=aarch64 run
# Run on loongarch64
make ARCH=loongarch64 run
```

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

### rCore-tutorial-v3 ch7

##### Prepare

```shell
git clone https://github.com/yfblock/rcore-tutorial-v3-with-hal-component.git
git reset fe2c146dedeadcc5fa9db8402128e066e45ca5a9 --hard
git clone https://github.com/Byte-OS/arch.git
```

##### Run on a specific platform

```shell
make ARCH=riscv64 run
```
