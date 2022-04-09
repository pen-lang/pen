#!/bin/sh

set -e

llvm_version=13

brew install jq mdbook ninja sccache

if [ $(uname) = Linux ]; then
  curl -fsSL https://apt.llvm.org/llvm.sh | sudo bash /dev/stdin 14
  llvm_prefix=/usr/lib/llvm-14
else
  brew install llvm@$llvm_version
  llvm_prefix=$(brew --prefix)/opt/llvm@$llvm_version
fi

echo LLVM_SYS_${llvm_version}0_PREFIX=$llvm_prefix >>$GITHUB_ENV
echo PATH=$llvm_prefix/bin:$PATH >>$GITHUB_ENV

if [ $(uname) = Linux ]; then
  sudo apt update --fix-missing
  sudo apt install libc6-dbg # for valgrind
  brew install valgrind
fi
