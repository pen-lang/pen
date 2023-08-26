#!/bin/sh

set -e

while getopts f option; do
  case $option in
  f)
    options=--fix
    ;;
  esac
done

shift $(expr $OPTIND - 1)

$(dirname $0)/run_all_crates.sh \
  cargo clippy $options --all-features -- \
  -D clippy::mod_module_files \
  -D clippy::use_self \
  "$@"
