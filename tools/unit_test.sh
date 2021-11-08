#!/bin/sh

set -ex

export RUST_MIN_STACK=8388608

for directory in . lib/os/ffi/application lib/os/ffi/library; do
  (
    cd $directory
    cargo test
  )
done
