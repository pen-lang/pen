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

build() {
  run_packages pen build

  (
    cd rust
    cargo bench --no-run
  )
}

run() {
  run_packages hyperfine -w 3 ./app

  (
    cd rust
    cargo bench -q
  )
}

prepare_integration_test $(dirname $PWD/$0)/..

cargo install hyperfine

cd $(dirname $0)/../benchmark

build
run
