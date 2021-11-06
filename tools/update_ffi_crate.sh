#!/bin/sh

set -e

root_directory=$(dirname $0)/..
ffi_version=$(grep 'version = ' lib/ffi/Cargo.toml | grep -o '[0-9.]\+' | head -n 1)

for directory in . cmd/test lib/os/ffi; do
  (
    cd $root_directory/$directory
    cargo update --aggressive -p pen-ffi:$ffi_version
  )
done
