#!/bin/sh

set -e

brew install llvm@12 ninja

llvm_prefix=$(brew --prefix)/opt/llvm@12

echo LLVM_SYS_120_PREFIX=$llvm_prefix >>$GITHUB_ENV
echo PATH=$llvm_prefix/bin:$PATH >>$GITHUB_ENV

if [ $RUNNER_OS = Linux ]; then
  sudo apt install libc6-dbg # for valgrind
  brew install valgrind
fi
