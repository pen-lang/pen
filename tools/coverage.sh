#!/bin/sh

set -e

. $(dirname $0)/utilities.sh

prepare_unit_test

rustup install nightly
rustup component add --toolchain nightly llvm-tools-preview

cargo install cargo-llvm-cov
cargo llvm-cov --json --output-path coverage.json
