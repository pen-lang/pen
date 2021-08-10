#!/bin/sh

set -ex

base_directory=$(dirname $0)/..

go run github.com/raviqqe/gherkin2markdown \
  $base_directory/features \
  $base_directory/doc/src/examples

curl -fsSL --retry 10 https://svgshare.com/i/ZvF.svg >$base_directory/doc/src/favicon.svg
