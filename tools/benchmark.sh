#!/bin/sh

set -e

. $(dirname $0)/utilities.sh

print_directory() {
  echo '>>>' "$@"
}

pen_packages() {
  for package_file in $(git ls-files | grep pen.json); do
    (
      directory=$(dirname $package_file)

      print_directory $directory
      cd $directory

      "$@"
    )
  done
}

rust_crates() {
  for cargo_file in $(git ls-files | grep Cargo.toml); do
    (
      directory=$(dirname $cargo_file)

      print_directory $directory
      cd $directory

      "$@"
    )
  done
}

benchmark() {
  hyperfine -w 3 "$@"
}

benchmark_rust() {
  command=$(basename $(pwd) | tr _ -)

  (
    cd $(cargo metadata --format-version 1 | jq -r .target_directory)/release

    benchmark ./$command
  )
}

build() {
  pen_packages pen build
  rust_crates cargo build --release
}

run() {
  pen_packages benchmark ./app
  rust_crates benchmark_rust
}

while getopts b option; do
  case $option in
  b)
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
