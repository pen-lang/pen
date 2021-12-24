#!/bin/sh

set -e

rustup install nightly
rustup component add --toolchain nightly llvm-tools-preview

cargo install cargo-llvm-cov
cargo llvm-cov --json --output-path coverage.json
