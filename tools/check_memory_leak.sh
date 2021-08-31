#!/bin/sh

set -ex

. $(dirname $0)/utilities.sh

if ! which valgrind; then
  "$@"
  exit
fi

valgrind --log-file=valgrind.log "$@"
test_valgrind_log valgrind.log
