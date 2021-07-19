#!/bin/sh

set -ex

base_directory=$(dirname $0)/..
document_directory=$base_directory/doc/content/examples

gherkin2markdown $base_directory/features $document_directory
rm -r $document_directory/smoke

for file in $(find $document_directory); do
  if ! echo $file | grep '\.md$' || [ $(basename $file) = '_index.md' ]; then
    continue
  fi

  title=$(grep '^# ' $file | sed 's/# //')

  cat >$file.tmp <<EOF
---
title: $title
---

EOF

  cat $file >>$file.tmp
  mv $file.tmp $file
done
