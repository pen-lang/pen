#!/bin/sh

set -ex

$(dirname $0)/run_all_crates.sh cargo test "$@"
