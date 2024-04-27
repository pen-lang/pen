#!/bin/sh

set -e

. $(dirname $0)/utilities.sh

cargo install cargo-machete

$(dirname $0)/run_all_crates.sh cargo machete
