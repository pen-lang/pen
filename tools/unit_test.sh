#!/bin/sh

set -e

export RUST_MIN_STACK=8388608

for directory in . lib/os/ffi/application lib/os/ffi/application; do
  (
    cd $directory
    cargo test
  )
done
