#!/bin/sh

set -ex

if [ $(uname) = Linux ]; then
  sudo apt update --fix-missing
  sudo apt install ninja-build
  sudo snap install --candidate --classic sccache
  curl -fsS https://apt.llvm.org/llvm.sh | sudo bash -s -- 13
else
  brew install llvm@13 ninja sccache

  llvm_prefix=$(brew --prefix llvm@13)

  echo export LLVM_SYS_130_PREFIX=$llvm_prefix >>$BASH_ENV
  echo export PATH=$llvm_prefix/bin:\$PATH >>$BASH_ENV
fi

curl -fsS https://sh.rustup.rs | sh -s -- -y
