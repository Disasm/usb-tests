#!/usr/bin/env bash

REPO=$(dirname $(readlink -f $0))
cd "$REPO"

set -euxo pipefail

for pkg in $(ls -d test-*); do
    cd "$REPO/$pkg"
    cargo build --tests
    cd fw
    cargo build --release --examples
done
