#!/bin/sh

set -e

export PATH=$PWD/target/release:$PWD/tools:$PATH
export RUSTC_WRAPPER=sccache
export PEN_ROOT=$PWD

cd $(dirname $0)/../benchmark

for package_file in $(git ls-files | grep pen.json); do
  (
    cd $(dirname $package_file)
    pen build
    hyperfine -w 3 ./app
  )
done
