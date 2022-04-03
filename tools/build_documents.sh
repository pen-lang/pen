#!/bin/sh

set -ex

base_directory=$(dirname $0)/..

generate_package_documentation() {
  (
    cd $base_directory/$1
    pen document $2
  )
}

generate_package_documentation lib/core Core
generate_package_documentation lib/os Os
generate_package_documentation lib/os-sync OsSync
generate_package_documentation lib/test Test

go run github.com/raviqqe/gherkin2markdown \
  $base_directory/features \
  $base_directory/doc/src/examples

curl -fsSL https://pen-lang.s3.us-west-1.amazonaws.com/icon.svg >$base_directory/doc/src/favicon.svg
