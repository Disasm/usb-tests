#!/usr/bin/env bash

REPO=$(dirname $(readlink -f $0))
cd "$REPO"

set -euxo pipefail

for pkg in $(ls -d test-*); do
    cd "$REPO/$pkg"
    cargo test --lib --target x86_64-unknown-linux-gnu  -- --test-threads=1
done
