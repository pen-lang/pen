#!/bin/sh

set -e

cd $(dirname $0)/ffi
cargo build --release --quiet
cp target/release/libprelude.a $1
