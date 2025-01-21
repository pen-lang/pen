#!/bin/sh

set -e

. $(dirname $0)/utilities.sh

prepare_integration_test $(dirname $PWD/$0)/..

for target in \
  i686-unknown-linux-musl \
  x86_64-unknown-linux-musl \
  aarch64-unknown-linux-musl \
  wasm32-wasip2; do
  rustup target add $target
done

bundler install

cd $(dirname $0)/..

cucumber --publish-quiet --strict-undefined "$@"
