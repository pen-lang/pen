#!/bin/sh

set -ex

. $(dirname $0)/utilities.sh

if ! which valgrind; then
  "$@"
  exit
fi

valgrind --log-file=valgrind.log "$@"

if ! test_valgrind_log valgrind.log; then
  cat valgrind.log
  exit 1
fi
