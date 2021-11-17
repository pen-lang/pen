#!/bin/sh

set -e

root_directory=$(dirname $0)/..

for directory in . cmd/test lib/os/ffi lib/os-async/ffi; do
  (
    cd $root_directory/$directory
    cargo update
  )
done
