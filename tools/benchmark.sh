#!/bin/sh

set -e

. $(dirname $0)/utilities.sh

run_packages() {
  for package_file in $(git ls-files | grep pen.json); do
    (
      directory=$(dirname $package_file)

      echo package: $directory
      cd $directory

      "$@"
    )
  done
}

run_rust() {
  (
    cd rust
    "$@"
  )
}

build() {
  run_packages pen build
  run_rust cargo bench --no-run
}

run() {
  run_packages hyperfine -w 3 ./app
  run_rust cargo bench -q
}

while getopts b option; do
  case $option in
  x)
    build=true
    ;;
  esac
done

shift $(expr $OPTIND - 1)

prepare_integration_test $(dirname $PWD/$0)/..

cargo install hyperfine

cd $(dirname $0)/../benchmark

build

if [ -z "$build" ]; then
  run
fi
