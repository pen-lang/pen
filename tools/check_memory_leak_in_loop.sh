#!/bin/sh

set -ex

. $(dirname $0)/utilities.sh

valgrind --log-file=valgrind.log "$@" >/dev/null &
pid=$!

sleep 5

kill $pid
wait $pid || :

test_valgrind_log valgrind.log
