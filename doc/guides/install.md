# Install

## Requirements

Pen requires the following software on your system.

- [`cargo`](https://github.com/rust-lang/cargo), the Rust package manager
- [`git`](https://git-scm.com/), the Git version control system
- [`ninja`](https://ninja-build.org/), the Ninja build system
- [`clang`](https://clang.llvm.org/), the LLVM-based C compiler
- [LLVM](https://llvm.org), the LLVM compiler infrastructure
  - Both the library and tools

### On Ubuntu

Run the following commands in your terminal to install the required software.
Note that we need to install LLVM from the external repository to get the specific version of it.

```sh
sudo apt install cargo git ninja-build
curl -fsSL https://apt.llvm.org/llvm.sh | sudo bash -s 12
```

### On macOS

To install `clang` and `llc`, install Xcode from the App Store.
Also, install the `cargo` and `git` commands via [Homebrew](https://brew.sh/) by running the following command in your terminal.

```sh
brew install git rust
```

## Installing the `pen` command

Run the following command in your terminal.

```sh
cargo install --git https://github.com/pen-lang/pen --branch main
```

Then, you should be able to run an `pen` command in your shell. Make sure that the `cargo`'s binary directory is included in your `PATH` environment variable.

```sh
pen --help
```
