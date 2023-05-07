#!/bin/sh

set -e

llvm_version=15

brew update
brew install jq llvm@$llvm_version ninja sccache

llvm_prefix=$(brew --prefix)/opt/llvm@$llvm_version

echo LLVM_SYS_${llvm_version}0_PREFIX=$llvm_prefix >>$GITHUB_ENV
echo PATH=$llvm_prefix/bin:$PATH >>$GITHUB_ENV

if [ $(uname) = Linux ]; then
  sudo apt update --fix-missing
  sudo apt install libc6-dbg # for valgrind
  brew install valgrind
fi
