# arch crate

> 支持多架构的 hal 层 crate.

English | [中文简体](./README.CN.md)

## Supported platforms

| Platform | Board |
| ---      |  ---  |
| riscv64  | qemu |
| x86_64   | qemu |
| aarch64  | qemu |
| loongarch64 | qemu |


## Example

在 example 文件夹中存在一个简单的 example.

### Run

``` shell
cd example
# 运行 riscv64 架构
make ARCH=riscv64 run
# 运行 x86_64 架构
make ARCH=x86_64 run
# 运行 aarch64 架构
make ARCH=aarch64 run
# 运行 loongarch64 架构
make ARCH=loongarch64 run
```
## Used OS

### ByteOS

下面介绍怎么运行

##### 环境准备

```shell
git clone https://github.com/Byte-OS/ByteOS.git
cd ByteOS
git reset 655eef3e38b5a85baaab4b2ba33832fbb299f19a --hard
git clone https://github.com/Byte-OS/arch.git
git reset cabff90bc4aecb7c8d3decb408c64e898112f6fe --hard
```

##### 运行在一个特定的平台

run on riscv64

```shell
make ARCH=riscv64 LOG=error run
```

如果想要运行在别的平台上，那么将 ARCH 改为其他值。

### rCore-tutorial-v3 ch7

##### 环境准备

```shell
git clone https://github.com/yfblock/rcore-tutorial-v3-with-hal-component.git
git reset fe2c146dedeadcc5fa9db8402128e066e45ca5a9 --hard
git clone https://github.com/Byte-OS/arch.git
git reset cabff90bc4aecb7c8d3decb408c64e898112f6fe --hard
```

##### 运行在一个特定平台上

```shell
make ARCH=riscv64 run
```

目前 tutorial 中存在一些 log, 并不影响运行。turorial 中可以使用测试程序 usertests 运行。
