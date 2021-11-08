#!/bin/sh

set -ex

for directory in . lib/os/ffi/application lib/os/ffi/library; do
  (
    cd $directory
    cargo test
  )
done
