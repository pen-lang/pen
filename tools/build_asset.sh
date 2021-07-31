#!/bin/sh

set -e

if [ $# -ne 1 ]; then
  exit 1
fi

version=$1
target=$(rustc -vV | grep host: | sed 's/host: //')
tarball=pen-$version-$target.tar.xz

cd $(dirname $0)/..

tar caf $tarball \
  README.md LICENSE.md LICENSE-MIT LICENSE-APACHE \
  lib \
  target/release/pen

echo $tarball
