#!/bin/sh

set -e

$(dirname $0)/build_documents.sh

mkdocs serve
