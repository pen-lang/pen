#!/bin/sh

set -ex

if [ $(uname) = Linux ]; then
  sudo apt update --fix-missing
  sudo apt install ninja-build
  sudo snap install --candidate --classic sccache
  curl -fsS https://apt.llvm.org/llvm.sh | sudo bash -s -- 13
else
  brew install ninja sccache
fi

curl -fsS https://sh.rustup.rs | sh -s -- -y
