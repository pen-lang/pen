#!/bin/sh

set -e

while getopts t: option; do
  case $option in
  t)
    target=$OPTARG
    ;;
  esac
done

shift $(expr $OPTIND - 1)

if [ -z $target ]; then
  exit 1
fi

rustup target add $target 2>/dev/null

cd $(dirname $0)/ffi
cargo build --release --quiet --target $target
cp target/$target/release/libcore.a $1
