#!/bin/sh

set -e

while getopts o:t: option; do
  case $option in
  o)
    output=$OPTARG
    ;;
  t)
    target=$OPTARG
    ;;
  esac
done

shift $(expr $OPTIND - 1)

if [ -z "$output" ]; then
  exit 1
elif [ -n "$target" ]; then
  target_option="-t $target"
fi

clang $target_option -o "$output" "$@" -ldl -lpthread
