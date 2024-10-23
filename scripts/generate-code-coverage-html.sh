#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

# Running those locally takes really long.
# Some tests `timeout` with Tarpaulin, although they do not when run with `cargo`.
# For this reason, use the --no-fail-fast flag.

cargo install cargo-tarpaulin
git submodule init
git submodule update
cargo tarpaulin --all --out Html --skip-clean --no-fail-fast
