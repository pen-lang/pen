#!/bin/sh

set -e

if [ $# -eq 0 ]; then
  exit 1
fi

root_directory=$(dirname $0)/..

for directory in . cmd/test lib/os/ffi lib/os-async/ffi; do
  (
    cd $root_directory/$directory
    "$@"
  )
done
