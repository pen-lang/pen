#!/bin/sh

set -e

rustup install nightly

alias cargo='rustup run nightly cargo'

cargo install cargo-llvm-cov
cargo llvm-cov
