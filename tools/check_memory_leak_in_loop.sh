#!/bin/sh

set -ex

. $(dirname $0)/utilities.sh

if ! which valgrind; then
  echo Valgrind not found! >&2
  "$@"
  exit
fi

test_duration=1
test_retry_count=3

test() {
  valgrind --log-file=valgrind.log "$@" >/dev/null &
  pid=$!

  sleep $test_duration

  kill $pid
  wait $pid || :

  test_valgrind_log valgrind.log
}

for _ in $(seq $test_retry_count); do
  if test "$@"; then
    break
  fi
done
