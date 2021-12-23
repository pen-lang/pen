#!/bin/sh

set -ex

$(dirname $0)/../../integration_test.sh \
  --format junit,fileattribute=true \
  --out /tmp/cucumber \
  $(circleci tests glob 'features/**/*.feature' |
    circleci tests split --split-by timings)
