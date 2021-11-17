#!/bin/sh

set -e

toolchain=nightly

rustup install $toolchain
rustup component add --toolchain $toolchain rustfmt

$(dirname $0)/run_all_crates.sh rustup run $toolchain cargo fmt "$@"
