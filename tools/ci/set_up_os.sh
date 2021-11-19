#!/bin/sh

set -e

brew install jq llvm ninja

echo PATH=$(brew --prefix)/opt/llvm/bin:$PATH >>$GITHUB_ENV

if [ $RUNNER_OS = Linux ]; then
  sudo apt install libc6-dbg # for valgrind
  brew install valgrind
fi
