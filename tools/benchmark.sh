#!/bin/sh

set -e

. $(dirname $0)/utilities.sh

prepare_language_environment $(dirname $PWD/$0)/..

echo $PEN_ROOT

cd $(dirname $0)/../benchmark

for package_file in $(git ls-files | grep pen.json); do
  (
    cd $(dirname $package_file)
    pen build
    hyperfine -w 3 ./app
  )
done
