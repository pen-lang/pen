#!/bin/sh

set -e

. $(dirname $0)/utilities.sh

brew install llvm@12 ninja

set_homebrew_llvm_environment_variables
