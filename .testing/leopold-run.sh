#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

./leopold-secrets-generation.sh

export CC=/usr/bin/clang && export CXX=/usr/bin/clang++ &&\
cargo run --color=always --features final -- --config $(pwd)/dist/authority.toml
