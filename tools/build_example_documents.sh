#!/bin/sh

set -ex

base_directory=$(dirname $0)/..

go run github.com/raviqqe/gherkin2markdown \
  $base_directory/features \
  $base_directory/doc/src/examples
