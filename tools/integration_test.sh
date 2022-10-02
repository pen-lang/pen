#!/bin/sh

set -e

. $(dirname $0)/utilities.sh

prepare_integration_test $(dirname $PWD/$0)/..

for target in \
  i686-unknown-linux-musl \
  x86_64-unknown-linux-musl \
  aarch64-unknown-linux-musl \
  wasm32-wasi; do
  rustup target add $target
done

bundler install

cd $(dirname $0)/..

tags=''

if [ $(uname) != Linux ]; then
  tags='not @linux'
fi

cucumber --publish-quiet --strict-undefined ${tags:+--tags "$tags"} "$@"
