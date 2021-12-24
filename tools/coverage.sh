#!/bin/sh

set -e

export RUSTFLAGS='-Z instrument-coverage'
export LLVM_PROFILE_FILE='json5format-%m.profraw'

rustup install nightly

rustup run nightly cargo test
rustup run nightly cargo profdata -- merge \
  -sparse json5format-*.profraw -o json5format.profdata
