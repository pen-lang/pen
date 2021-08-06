#!/bin/sh

set -e

cat $(dirname $0)/../cmd/pen/Cargo.toml | grep 'version = ' | cut -f 2 -d '"'
