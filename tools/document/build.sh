#!/bin/sh

set -ex

document_directory=doc/src/content/docs
package_document_directory=$document_directory/references/standard-packages
example_directory=$document_directory/examples

prepend_title() {
  temporary_file=$1.tmp

  (
    echo ---
    echo "title: $2"
    echo ---
    sed 1d $1
  ) >$temporary_file

  mv $temporary_file $1
}

build_package_document() {
  file=$package_document_directory/$1.md

  (
    cd packages/$1
    pen document \
      --name $2 \
      --url pen:///$1 \
      --description "$3"
  ) >$file

  prepend_title $file $2
}

cd $(dirname $0)/../..

tools/build.sh

export PATH=$PWD/target/release:$PATH

rm -f $package_document_directory/*.md

build_package_document \
  core \
  Core \
  "This package provides common algorithms and data structures."

build_package_document \
  flag \
  Flag \
  "This package provides command-line flag parsing."

build_package_document \
  html \
  Html \
  "This package provides HTML rendering logic."

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
  random \
  Random \
  "This package provides random number generation."

build_package_document \
  reflect \
  Reflect \
  "This package provides reflection."

build_package_document \
  regex \
  Regex \
  "This package provides regular expressions."

build_package_document \
  sql \
  Sql \
  "This package provides a SQL database client."

build_package_document \
  test \
  Test \
  "This package provides test utilities."

rm -rf $example_directory/*

go tool gherkin2markdown features $example_directory

rm -r $example_directory/smoke

for file in $(find $example_directory -name '*.md'); do
  prepend_title $file "$(sed -n '1s/^# //p' $file)"
done

(
  cd doc

  pnpm install --frozen-lockfile
  pnpm build
)
