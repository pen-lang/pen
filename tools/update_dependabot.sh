#!/bin/sh

set -e

(
  cat <<EOF
version: 2
updates:
  - package-ecosystem: github-actions
    directories:
      - /
      - /.github/actions/*
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
  - package-ecosystem: rust-toolchain
    directories:
      - /
EOF

  for file in $(git ls-files '**/rust-toolchain.toml'); do
    echo "      - /$(dirname $file)"
  done

  cat <<EOF
    groups:
      rust-toolchain:
        patterns:
          - "*"
    schedule:
      interval: daily
  - package-ecosystem: uv
    directory: /doc
    schedule:
      interval: daily
  - package-ecosystem: cargo
    directories:
      - /
EOF

  for file in $(git ls-files '**/Cargo.lock'); do
    echo "      - /$(dirname $file)"
  done

  cat <<EOF
    groups:
      cargo:
        patterns:
          - "*"
      cargo-security:
        applies-to: security-updates
        patterns:
          - "*"
    schedule:
      interval: daily
EOF
) >$(dirname $0)/../.github/dependabot.yaml
