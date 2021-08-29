#!/bin/sh

set -e

cd $(dirname $0)/..

PEN_ROOT=$PWD PATH=$PWD/target/release:$PWD/tools:$PATH cucumber --publish-quiet "$@"
