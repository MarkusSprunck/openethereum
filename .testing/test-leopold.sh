#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error


cd ..
export CC=$(which gcc-12) &&\
export CXX=$(which g++-12) &&\
cargo run --color=always --release --features final -- --config /workspaces/openethereum/.testing/dist/authority.toml