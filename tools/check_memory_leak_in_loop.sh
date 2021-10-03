#!/bin/sh

set -ex

. $(dirname $0)/utilities.sh

test_duration=1
test_retry_count=5

test() {
  valgrind=""

  if which valgrind; then
    valgrind="valgrind --log-file=valgrind.log"
  fi

  $valgrind "$@" >/dev/null &
  pid=$!

  sleep $test_duration

  kill $pid
  wait $pid || :

  if which valgrind; then
    test_valgrind_log valgrind.log
  fi
}

retry_tests() {
  for _ in $(seq $test_retry_count); do
    if test "$@"; then
      return 0
    fi

    echo Retrying...
  done

  return 1
}

retry_tests "$@"
