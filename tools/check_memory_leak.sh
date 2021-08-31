#!/bin/sh

set -ex

. $(dirname $0)/utilities.sh

check_valgrind_command "$@"

valgrind --log-file=valgrind.log "$@"
test_valgrind_log valgrind.log
