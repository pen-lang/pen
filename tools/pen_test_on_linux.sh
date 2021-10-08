#!/bin/sh

# TODO Rmove this script when pen test with multiple tests is fixed on macOS.

set -e

if llvm-config --host-target | grep apple; then
  exit
fi

pen test
