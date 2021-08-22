# Cross compile

The language's compiler supports [cross compile](https://en.wikipedia.org/wiki/cross_compiler). To compile applications and libraries for different targets, specify the `--target` option of the `pen build` subcommand.

For example, run the following command to compile a `wasm32` binary for the WASI platform.

```sh
pen build --target wasm32-wasi
```

Note that we currently support those targets via [Rust](https://www.rust-lang.org/)'s cross compiler toolchain. Please install Rust compiler through [`rustup`](https://rust-lang.github.io/rustup/) and refer to [its documentation](https://rust-lang.github.io/rustup/cross-compilation.html) on how to install cross compiler toolchains for specific targets.

For example, if you want to cross-compile binaries for and install a toolchain of a `wasm32-wasi` target, run the following command.

```sh
rustup target add wasm32-wasi
```

## Supported targets

Run `pen build --help` to see all supported targets.

## System package support

Cross compile support of system packages are totally up to their developers.

For example, [the standard system package of `Os`](/references/standard-packages/os.md) supports most targets as long as their platforms expose OS-like APIs. However, some custom system packages might not be worth suppporting all those targets because of their limited use cases.
