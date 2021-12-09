#!/bin/sh

set -e

brew install jq llvm@13 ninja sccache

if [ $(uname) = Linux ]; then
  sudo apt install libc6-dbg # for valgrind
  brew install valgrind
fi
