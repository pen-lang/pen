#!/bin/sh

set -e

cargo install cargo-llvm-cov
cargo llvm-cov --json --output-path coverage.json
