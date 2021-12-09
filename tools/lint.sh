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

. $(dirname $0)/utilities.sh

install_nightly_component clippy

# TODO Enable -D clippy::use_self.
$(dirname $0)/run_all_crates.sh \
  rustup run nightly cargo clippy $options -Z unstable-options -- \
  -D clippy::mod_module_files \
  "$@"
