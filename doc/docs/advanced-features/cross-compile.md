# Cross compile

The language's compiler supports [cross compile](https://en.wikipedia.org/wiki/cross_compiler). To compile applications and libraries for different targets, specify the `--target` option of the `pen build` subcommand.

For example, run the following command to compile a `wasm32` binary for the WASI platform.

```sh
pen build --target wasm32-wasip2
```

Note that we currently support those targets via [Rust](https://www.rust-lang.org/)'s cross compiler toolchain. Please install a Rust compiler through [`rustup`](https://rust-lang.github.io/rustup/) to enable installation of toolchains for different targets.

## Supported targets

Run `pen build --help` to see all supported targets.

## System package support

Cross compile support of [system packages](../references/language/packages.md#system-packages) are totally up to their developers. For example, [the `Os` standard system package](../references/standard-packages/os.md) supports most targets as long as their platforms expose OS-like APIs. However, some custom system packages might not support those targets because of their limited use cases.
