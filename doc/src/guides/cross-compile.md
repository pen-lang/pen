# Cross compile

The language's compiler supports [cross compile](https://en.wikipedia.org/wiki/cross_compiler). To compile applications and libraries for different targets, specify the `--target` option of the `pen build` subcommand.

For example, run the following command to compile a `wasm32` binary for the WASI platform.

```sh
pen build --target wasm32-wasi
```

## Supported targets

Run `pen build --help` to see all supported targets.

## System package support

Cross compile support of system packages are totally up to their developers.

For example, [the standard system package of `Os`](/references/standard-packages/os.md) supports most targets as long as their platforms expose OS-like APIs.
