#!/bin/sh

set -e

$(dirname $0)/build.sh

cd doc
mkdocs serve
