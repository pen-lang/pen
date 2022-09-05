#!/bin/sh

set -e

$(dirname $0)/build.sh

mkdocs serve
