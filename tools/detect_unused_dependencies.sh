#!/bin/sh

set -e

. $(dirname $0)/utilities.sh

$(dirname $0)/run_all_crates.sh rustup run nightly cargo udeps
