#!/bin/sh

set -ex

. $(dirname $0)/utilities.sh

valgrind --log-file=valgrind.log "$@"
test_valgrind_log valgrind.log
