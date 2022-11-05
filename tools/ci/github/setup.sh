#!/bin/sh

set -e

llvm_version=14

brew update
brew install jq llvm@$llvm_version ninja sccache

llvm_prefix=$(brew --prefix)/opt/llvm@$llvm_version

echo LLVM_SYS_${llvm_version}0_PREFIX=$llvm_prefix >>$GITHUB_ENV
echo PATH=$llvm_prefix/bin:$PATH >>$GITHUB_ENV

case $(uname) in
Darwin)
  rustup install stable

  for component in rls; do
    rustup component add $component --toolchain stable-x86_64-apple-darwin
  done
  ;;
Linux)
  sudo apt update --fix-missing
  sudo apt install libc6-dbg # for valgrind
  brew install valgrind
  ;;
esac
