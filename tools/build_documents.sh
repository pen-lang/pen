#!/bin/sh

set -ex

base_directory=$(dirname $0)/..
document_directory=$base_directory/doc/src/references/standard-packages

generate_package_documentation() {
  (
    cd $base_directory/$1
    pen document $2
  )
}

generate_package_documentation lib/core Core >$document_directory/core.md
generate_package_documentation lib/os Os >$document_directory/os.md
generate_package_documentation lib/os-sync OsSync >$document_directory/os-sync.md
generate_package_documentation lib/test Test >$document_directory/test.md

go run github.com/raviqqe/gherkin2markdown \
  $base_directory/features \
  $base_directory/doc/src/examples

curl -fsSL https://pen-lang.s3.us-west-1.amazonaws.com/icon.svg >$base_directory/doc/src/favicon.svg
