#!/bin/sh

set -ex

base_directory=$(dirname $0)/..

go run github.com/raviqqe/gherkin2markdown \
  $base_directory/features \
  $base_directory/doc/src/examples

curl -fsSL https://pen-lang.s3.us-west-1.amazonaws.com/icon.svg >$base_directory/doc/src/favicon.svg
