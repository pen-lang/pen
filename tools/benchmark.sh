#!/bin/sh

set -e

. $(dirname $0)/utilities.sh

prepare_integration_test $(dirname $PWD/$0)/..

cd $(dirname $0)/../benchmark

for package_file in $(git ls-files | grep pen.json); do
  (
    directory=$(dirname $package_file)

    echo $directory
    cd $directory

    pen build

    hyperfine -w 3 ./app
  )
done
