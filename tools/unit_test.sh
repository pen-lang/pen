#!/bin/sh

set -ex

export RUST_MIN_STACK=8388608

$(dirname $0)/run_all_crates.sh cargo test "$@"
