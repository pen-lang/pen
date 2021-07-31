#!/bin/sh

set -e

if [ $# -ne 1 ]; then
  exit 1
fi

version=$1
base_directory=$(cd $(dirname $0) && pwd)/..
asset_directory=$base_directory/tmp/asset
target=$(rustc -vV | grep host: | sed 's/host: //')
tarball=$base_directory/pen-$version-$target.tar.gz

rm -rf $asset_directory
mkdir -p $asset_directory

(
  cd $base_directory

  cp -r README.md LICENSE.md LICENSE-MIT LICENSE-APACHE lib $asset_directory

  mkdir $asset_directory/bin
  cp target/release/pen $asset_directory/bin
)

(
  cd $asset_directory
  tar czf $tarball *
)

echo $tarball
