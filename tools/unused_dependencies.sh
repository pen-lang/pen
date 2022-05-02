#!/bin/sh

set -e

. $(dirname $0)/utilities.sh

version=nightly-2022-04-25

rustup install $version
rustup run $version cargo install cargo-udeps

$(dirname $0)/run_all_crates.sh rustup run $version cargo udeps
