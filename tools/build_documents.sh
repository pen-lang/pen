#!/bin/sh

set -ex

document_directory=doc/src/references/standard-packages

build_package_document() {
  (
    cd packages/$1
    pen document \
      --name $2 \
      --url pen:///$1 \
      --description "$3"
  ) >$document_directory/$1.md
}

cd $(dirname $0)/..

tools/build.sh

export PATH=$PWD/target/release:$PATH

build_package_document \
  core \
  Core \
  "This package provides common algorithms and data structures."

build_package_document \
  flag \
  Flag \
  "This package provides command-line flag parsing."

build_package_document \
  http \
  Http \
  "This package provides HTTP client and server."

build_package_document \
  json \
  Json \
  "This package provides a JSON parser."

build_package_document \
  os \
  Os \
  "This package provides an interface for operating systems."

build_package_document \
  sql \
  Sql \
  "This package provides a SQL database client."

build_package_document \
  test \
  Test \
  "This package provides test utilities."

go run github.com/raviqqe/gherkin2markdown features doc/src/examples

curl -fsSL https://pen-lang.s3.us-west-1.amazonaws.com/icon.svg >doc/src/favicon.svg

# spell-checker: disable-next-line
cargo install mdbook-pagetoc

cd doc
mdbook build
