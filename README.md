# arch crate

> A crate help you to write a os that support multiple platforms.

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
# Run on riscv64
make ARCH=riscv64 example
# Run on x86_64
make ARCH=x86_64 example
# Run on aarch64
make ARCH=aarch64 example
# Run on loongarch64
make ARCH=loongarch64 example
```
