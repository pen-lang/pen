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

if [ -n "$target" ]; then
  target_option="-t $target"
fi

clang $target_option -o "$@" -ldl -lpthread
