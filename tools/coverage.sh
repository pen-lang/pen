#!/bin/sh

set -e

. $(dirname $0)/utilities.sh

prepare_unit_test

toolchain=nightly-2022-03-01

rustup install $toolchain
rustup component add --toolchain $toolchain llvm-tools-preview

cargo install cargo-llvm-cov
cargo +$toolchain llvm-cov --workspace --lcov --output-path lcov.info
