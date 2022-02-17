#!/bin/sh

set -e

. $(dirname $0)/utilities.sh

cargo install cargo-udeps

$(dirname $0)/run_all_crates.sh rustup run nightly cargo udeps
