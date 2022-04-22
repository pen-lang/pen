#!/bin/sh

set -ex

if [ $# -eq 0 ]; then
  exit 1
fi

package_ffi_directories() {
  for file in $(find packages -name Cargo.lock); do
    dirname $file
  done
}

cd $(dirname $0)/..

for directory in . $(package_ffi_directories); do
  (
    cd $directory
    "$@"
  )
done

(
  cd cmd/test
  PEN_ARCHIVE_FILES= "$@"
)
