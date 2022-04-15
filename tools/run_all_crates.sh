#!/bin/sh

set -ex

if [ $# -eq 0 ]; then
  exit 1
fi

root_directory=$(dirname $0)/..

for directory in \
  . \
  lib/core/ffi \
  lib/http/ffi \
  lib/json/ffi \
  lib/os/ffi \
  lib/os-sync/ffi \
  lib/prelude/ffi; do
  (
    cd $root_directory/$directory
    "$@"
  )
done

(
  cd $root_directory/cmd/test
  PEN_ARCHIVE_FILES= "$@"
)
