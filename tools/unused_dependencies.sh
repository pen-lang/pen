#!/bin/sh

set -e

. $(dirname $0)/utilities.sh

rustup install nightly
rustup run nightly cargo install cargo-udeps

$(dirname $0)/run_all_crates.sh rustup run nightly cargo udeps
