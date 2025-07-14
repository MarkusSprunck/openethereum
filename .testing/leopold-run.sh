#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

echo "_____ Use folder _____"
if [ "$(basename "$PWD")" == "openethereum" ]; then
  cd .testing
fi
echo "$PWD"

echo "_____ Set Rust Version _____"
rustup override set 1.88

echo "_____ Run Leopold _____"
export CC=/usr/bin/clang && export CXX=/usr/bin/clang++ &&\
cargo run --color=always --features final -- --config $(pwd)/dist/authority.toml
