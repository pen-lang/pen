#!/bin/sh

set -e

$(dirname $0)/run_all_crates.sh \
  rustup run nightly cargo clippy --fix -Z unstable-options -- \
  -D warnings \
  -D clippy::use_self \
  -D clippy::mod_module_files
