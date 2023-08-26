#!/bin/sh

set -e

. $(dirname $0)/utilities.sh

# spell-checker: disable-next-line
install_nightly_component rustfmt

$(dirname $0)/run_all_crates.sh rustup run nightly cargo fmt --all-features "$@"
