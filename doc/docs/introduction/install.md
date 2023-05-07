# Install

## Via Homebrew

On Linux, macOS and [WSL](https://docs.microsoft.com/en-us/windows/wsl/about), you can install Pen through [Homebrew][homebrew].

1.  Install [Homebrew][homebrew].
1.  Run the following command in your terminal.

    ```sh
    brew install pen-lang/pen/pen
    ```

Now, you should be able to run a `pen` command in your shell.

```sh
pen --version
```

## Building from source

You can also build Pen from source on your local machine.

1. Install the following software using a package manager of your choice (e.g. `apt` for Ubuntu and [Homebrew][homebrew] for macOS.)

   - [Rust](https://www.rust-lang.org/)
   - [LLVM 15](https://llvm.org/)
   - [Git](https://git-scm.com/)
   - [Ninja](https://ninja-build.org/)

1. Clone the Git repository.

   ```sh
   git clone https://github.com/pen-lang/pen
   ```

1. Run a `cargo` command in the repository's directory.

   ```sh
   cargo install --path cmd/pen
   ```

1. Set a `PEN_ROOT` environment variable to the directory.

   ```sh
   export PEN_ROOT=<directory>
   ```

Now, you are ready to use the `pen` command built manually.

[homebrew]: https://brew.sh
