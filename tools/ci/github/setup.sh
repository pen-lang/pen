#!/bin/sh

set -e

brew install jq ninja sccache

echo PATH=$llvm_prefix/bin:$PATH >>$GITHUB_ENV

if [ $(uname) = Linux ]; then
  curl -f https://apt.llvm.org/llvm.sh | sudo bash /dev/stdin 14

  sudo apt update --fix-missing
  sudo apt install libc6-dbg # for valgrind
  brew install valgrind
fi
