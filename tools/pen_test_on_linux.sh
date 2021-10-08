#!/bin/sh

# TODO Rmove this script when pen test with multiple tests is fixed on macOS.

set -e

if [ $(uname) = Darwin ]; then
  exit
fi

pen test
