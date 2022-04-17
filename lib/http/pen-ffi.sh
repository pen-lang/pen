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

cd $(dirname $0)/ffi
cargo build --release --quiet --target $target
# spell-checker: disable-next-line
cp target/$target/release/libhttp.a $1
