# For developers

## Building from source

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

1. You are ready to build packages with the customized `pen` command and libraries!
