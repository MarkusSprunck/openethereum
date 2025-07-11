#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

./leopold-secrets-generation.sh

echo "_____ Set Rust Version _____"
rustup override set 1.86

echo "_____ Run Leopold _____"
export CC=/usr/bin/clang && export CXX=/usr/bin/clang++ &&\
cargo run --color=always --features final -- --config $(pwd)/dist/authority.toml
