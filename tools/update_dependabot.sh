#!/bin/sh

set -e

(
  cat <<EOF
version: 2
updates:
  - package-ecosystem: github-actions
    directory: /
    schedule:
      interval: daily
  - package-ecosystem: gomod
    directory: /
    schedule:
      interval: daily
  - package-ecosystem: bundler
    directory: /
    schedule:
      interval: daily
  - package-ecosystem: npm
    directory: /doc
    schedule:
      interval: daily
  - package-ecosystem: pip
    directory: /doc
    schedule:
      interval: daily
  - package-ecosystem: cargo
    directory: /
    schedule:
      interval: daily
EOF

  for file in $(git ls-files '**/Cargo.lock'); do
    cat <<EOF
  - package-ecosystem: cargo
    directory: /$(dirname $file)
    schedule:
      interval: daily
EOF
  done
) >$(dirname $0)/../.github/dependabot.yml
