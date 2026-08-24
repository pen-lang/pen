#!/bin/sh

set -e

$(dirname $0)/build.sh

cd $(dirname $0)/../../doc

pnpm dev "$@"
