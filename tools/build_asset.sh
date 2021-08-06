#!/bin/sh

set -e

version=$($(dirname $0)/version.sh)
target=$(rustc -vV | grep host: | sed 's/host: //')
tarball=pen-$version-$target.tar.xz

cd $(dirname $0)/..

tar caf $tarball \
  README.md LICENSE.md LICENSE-MIT LICENSE-APACHE \
  doc lib target/release/pen

echo $tarball
