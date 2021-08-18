#!/bin/sh

set -e

curl -fsSL https://apt.llvm.org/llvm.sh | sudo bash -s 12
sudo apt install clang ninja-build valgrind zsh
