#!/bin/sh

set -ex

. $(dirname $0)/utilities.sh

prepare_unit_test

$(dirname $0)/run_all_crates.sh cargo test "$@"
