#!/bin/sh

set -ex

base_directory=$(dirname $0)/..
document_directory=$base_directory/doc/src/references/standard-packages

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
    --description "This package provides operations in operating systems."
) >$document_directory/os.md

(
  cd $base_directory/lib/os-sync
  pen document \
    --name OsSync \
    --url pen:///os-sync \
    --description 'This package provides operations in operating systems. The package provides similar but synchronous functions differently from an `Os` package.'
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
