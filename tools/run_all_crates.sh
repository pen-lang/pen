#!/bin/sh

set -ex

if [ $# -eq 0 ]; then
  exit 1
fi

root_directory=$(dirname $0)/..

for directory in \
  . \
  packages/core/ffi \
  packages/http/ffi \
  packages/json/ffi \
  packages/os/ffi/application \
  packages/os/ffi/library \
  packages/os-sync/ffi \
  packages/prelude/ffi; do
  (
    cd $root_directory/$directory
    "$@"
  )
done

(
  cd $root_directory/cmd/test
  PEN_ARCHIVE_FILES= "$@"
)
