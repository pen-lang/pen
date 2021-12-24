#!/bin/sh

set -e

export RUSTFLAGS='-Z instrument-coverage'
export LLVM_PROFILE_FILE=%m.prof.raw

alias cargo='rustup run nightly cargo'

rustup install nightly
rustup component add --toolchain nightly llvm-tools-preview
cargo install cargo-binutils

cargo test
cargo profdata -- merge -sparse $(find -name '*.prof.raw') -o coverage.prof
cargo cov -- show -instr-profile=coverage.prof >coverage.txt
