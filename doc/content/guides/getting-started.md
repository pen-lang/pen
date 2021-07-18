---
title: Getting started
weight: 1
---

# Getting started

## Install

### Requirements

Pen requires the following software on your system.

- [`cargo`](https://github.com/rust-lang/cargo), the Rust package manager
- [`git`](https://git-scm.com/), the Git version control system
- [`ninja`](https://ninja-build.org/), the Ninja build system
- [`clang`](https://clang.llvm.org/), the LLVM-based C compiler
- [LLVM](https://llvm.org), the LLVM compiler infrastructure
  - Both the library and tools

#### On Ubuntu

Run the following commands in your terminal to install the required software.
Note that we need to install LLVM from the external repository to get the specific version of it.

```sh
sudo apt install cargo git ninja-build
curl -fsSL https://apt.llvm.org/llvm.sh | sudo bash -s 12
```

#### On macOS

To install `clang` and `llc`, install Xcode from the App Store.
Also, install the `cargo` and `git` commands via [Homebrew](https://brew.sh/) by running the following command in your terminal.

```sh
brew install git rust
```

### Installing the `pen` command

Run the following command in your terminal.

```sh
cargo install --git https://github.com/pen-lang/pen --branch main
```

Then, you should be able to run an `pen` command in your shell. Make sure that the `cargo`'s binary directory is included in your `PATH` environment variable.

```sh
pen --help
```

## Creating a package

To create your first package, run the following command.

```sh
pen create foo
```

Then, you should see a `foo` directory under your current directory. When you switch to the `foo` directory, you should see a `Main.pen` source file and a `pen.json` package configuration file there.

## Building a package

To build the package, run the following command in the `foo` directory.

```sh
pen build
```

Then, you will see an executable file named `app` in the directory. Run the command to see your first "Hello, world!"

```sh
./app
```

## For more information...

Now, you can start editing `*.pen` files and build your own application!

- To know more about the language, see [the language reference](/references/language).
