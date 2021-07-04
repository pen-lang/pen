#!/bin/sh

set -e

cd $(dirname $0)/ffi
cargo build --release --quiet
cp target/release/libos.a $1
