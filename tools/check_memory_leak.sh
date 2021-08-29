#!/bin/sh

set -ex

. $(dirname $0)/utilities.sh

valgrind "$@"
test_valgrind_log valgrind.log
