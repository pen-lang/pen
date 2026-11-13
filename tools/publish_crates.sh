#!/bin/sh

set -e

[ -n "$CI" ]

cargo install cargo-workspaces
cargo workspaces publish -y --publish-as-is "$@"
