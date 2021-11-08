#!/bin/sh

set -e

for directory in . lib/os/ffi/application lib/os/ffi/application; do
  (
    cd $directory
    cargo test
  )
done
