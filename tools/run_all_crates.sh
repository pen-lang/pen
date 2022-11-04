#!/bin/sh

set -ex

if [ $# -eq 0 ]; then
  exit 1
fi

cd $(dirname $0)/..

for file in $(git ls-files 'packages/**/Cargo.lock'); do
  dirname $file
  (
    cd $(dirname $file)
    "$@"
  )
done

(
  cd cmd/test
  PEN_ARCHIVE_FILES= "$@"
)
