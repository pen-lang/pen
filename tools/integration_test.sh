#!/bin/sh

set -e

for target in \
  i686-unknown-linux-musl \
  x86_64-unknown-linux-musl \
  aarch64-unknown-linux-musl \
  wasm32-wasi; do
  rustup target add $target
done

bundler install

cargo install turtle-build

cd $(dirname $0)/..

export PATH=$PWD/target/release:$PWD/tools:$PATH
export RUSTC_WRAPPER=sccache
export PEN_ROOT=$PWD

# TODO Do not retry. The option is set currently for:
# - https://github.com/pen-lang/pen/issues/578
cucumber --publish-quiet --retry 5 "$@"
