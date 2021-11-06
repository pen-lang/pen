#!/bin/sh

set -e

root_directory=$(dirname $0)/..

for directory in . cmd/test lib/os/ffi; do
  (
    cd $root_directory/$directory
    cargo update --aggressive -p pen-ffi
  )
done
