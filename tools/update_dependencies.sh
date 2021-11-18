#!/bin/sh

set -e

$(dirname $0)/run_all_crates.sh cargo update
