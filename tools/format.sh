#!/bin/sh

set -e

. $(dirname $0)/utilities.sh

install_nightly_component rustfmt

$(dirname $0)/run_all_crates.sh rustup run nightly cargo fmt "$@"
