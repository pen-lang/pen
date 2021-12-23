#!/bin/sh

set -ex

sudo apt update --fix-missing
sudo apt install ninja-build
sudo snap install --candidate --classic sccache
curl -fsS https://apt.llvm.org/llvm.sh | sudo bash -s -- 13
curl -fsS https://sh.rustup.rs | sh -s -- -y
