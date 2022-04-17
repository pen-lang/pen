#!/bin/sh

set -ex

document_directory=doc/src/references/standard-packages

os_package_description() {
  echo This package provides an interface for operating systems.
}

os_sync_package_description() {
  os_package_description

  echo Functions in this package are synchronous differently from an '`Os`' package.
}

cd $(dirname $0)/..

tools/build.sh

export PATH=$PWD/target/release:$PATH

(
  cd lib/core
  pen document \
    --name Core \
    --url pen:///core \
    --description "This package provides common algorithms and data structures."
) >$document_directory/core.md

(
  cd lib/flag
  pen document \
    --name Flag \
    --url pen:///flag \
    --description "This package provides command-line flag parsing."
) >$document_directory/flag.md

(
  cd lib/http
  pen document \
    --name Http \
    --url pen:///http \
    --description "This package provides HTTP client and server."
) >$document_directory/http.md

(
  cd lib/json
  pen document \
    --name Json \
    --url pen:///json \
    --description "This package provides a JSON parser."
) >$document_directory/json.md

(
  cd lib/os
  pen document \
    --name Os \
    --url pen:///os \
    --description "$(os_package_description)"
) >$document_directory/os.md

(
  cd lib/test
  pen document \
    --name Test \
    --url pen:///test \
    --description "This package provides test utilities."
) >$document_directory/test.md

go run github.com/raviqqe/gherkin2markdown features doc/src/examples

curl -fsSL https://pen-lang.s3.us-west-1.amazonaws.com/icon.svg >doc/src/favicon.svg

(
  cd doc
  mdbook build
)
