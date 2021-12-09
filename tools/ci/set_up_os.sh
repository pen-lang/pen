#!/bin/sh

set -e

brew install jq llvm@13 ninja sccache

if [ $(uname) = Linux ]; then
  brew install valgrind
fi
