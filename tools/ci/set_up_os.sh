#!/bin/sh

set -e

if [ $# -ne 1 ]; then
  exit 1
fi

os=$1

brew install llvm@12 ninja

llvm_prefix=$(brew --prefix)/opt/llvm@12

echo LLVM_SYS_120_PREFIX=$llvm_prefix >>$GITHUB_ENV
echo PATH=$llvm_prefix/bin:$PATH >>$GITHUB_ENV

if echo $os | grep ubuntu; then
  brew install valgrind
fi
