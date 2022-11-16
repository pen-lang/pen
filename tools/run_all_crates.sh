#!/bin/sh

set -ex

if [ $# -eq 0 ]; then
  exit 1
fi

cd $(dirname $0)/..

for file in $(git ls-files Cargo.lock '**/Cargo.lock'); do
  (
    cd $(dirname $file)
    "$@"
  )
done
