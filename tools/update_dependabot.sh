#!/bin/sh

set -e

cat >$(dirname $0)/../.github/dependabot.yaml <<EOF
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
      - /packages/*/ffi
      - /packages/os/ffi/*
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
      - /cmd/test
      - /packages/*/ffi
      - /packages/os/ffi/*
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
