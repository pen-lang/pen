#!/bin/sh

set -e

for channel in stable nightly; do
  rustup install $channel

  for target in \
    i686-unknown-linux-musl \
    x86_64-unknown-linux-musl \
    aarch64-unknown-linux-musl \
    wasm32-wasi; do
    rustup target add --toolchain $channel $target
  done
done

cd $(dirname $0)/..

PEN_ROOT=$PWD PATH=$PWD/target/release:$PWD/tools:$PATH cucumber --publish-quiet "$@"
