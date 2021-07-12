#!/bin/sh

set -e

cd $(dirname $0)/ffi
cargo build --release --quiet
cp target/release/libcore.a $1
