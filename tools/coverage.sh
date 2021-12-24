#!/bin/sh

set -e

rustup install nightly

cargo install cargo-llvm-cov
cargo llvm-cov --json --output-path coverage.json
