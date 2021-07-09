#!/bin/sh

set -ex

. $(dirname $0)/utilities.sh

valgrind --log-file=valgrind.log "$@" >/dev/null
test_valgrind_log valgrind.log
