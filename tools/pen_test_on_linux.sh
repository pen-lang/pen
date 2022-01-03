#!/bin/sh

# TODO Remove this script when pen test with multiple tests is fixed on macOS.

set -e

if [ $(uname) = Darwin ]; then
  exit
fi

pen test
