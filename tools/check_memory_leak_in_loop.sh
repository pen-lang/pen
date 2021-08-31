#!/bin/sh

set -ex

. $(dirname $0)/utilities.sh

test_duration=1
test_retry_count=3

test() {
  command="valgrind --log-file=valgrind.log $@"

  if ! which valgrind; then
    command="$@"
  fi

  $command >/dev/null &
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
