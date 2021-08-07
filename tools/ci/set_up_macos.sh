#!/bin/sh

set -e

brew install coreutils ninja llvm@12

llvm_prefix=$(brew --prefix)/opt/llvm@12

echo LLVM_SYS_120_PREFIX=$llvm_prefix >>$GITHUB_ENV
echo PATH=$llvm_prefix/bin:$PATH >>$GITHUB_ENV
