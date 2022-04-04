#!/bin/sh

set -ex

base_directory=$(dirname $0)/..
document_directory=$base_directory/doc/src/references/standard-packages

os_package_description() {
  echo This package provides types and functions to interact with operating systems.
}

os_sync_package_description() {
  os_package_description

  echo Functions in this package are synchronous differently from an '`Os`' package.
}

(
  cd $base_directory/lib/core
  pen document \
    --name Core \
    --url pen:///core \
    --description "This package provides common algrorithms and data structures."
) >$document_directory/core.md

(
  cd $base_directory/lib/os
  pen document \
    --name Os \
    --url pen:///os \
    --description "$(os_package_description)"
) >$document_directory/os.md

(
  cd $base_directory/lib/os-sync
  pen document \
    --name OsSync \
    --url pen:///os-sync \
    --description "$(os_sync_package_description)"
) >$document_directory/os-sync.md

(
  cd $base_directory/lib/test
  pen document \
    --name Test \
    --url pen:///test \
    --description "This package provides test utilities."
) >$document_directory/test.md

go run github.com/raviqqe/gherkin2markdown \
  $base_directory/features \
  $base_directory/doc/src/examples

curl -fsSL https://pen-lang.s3.us-west-1.amazonaws.com/icon.svg >$base_directory/doc/src/favicon.svg
