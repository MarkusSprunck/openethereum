#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

cd ..
export CC=/usr/bin/clang && export CXX=/usr/bin/clang++ &&\
cargo run --color=always --features final -- --config $(pwd)/.testing/dist/authority.toml
