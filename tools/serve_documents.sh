#!/bin/sh

set -e

$(dirname $0)/build_documents.sh

(
  cd $(dirname $0)/../doc
  mdbook serve "$@"
)
