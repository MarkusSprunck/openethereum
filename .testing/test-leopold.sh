#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

echo "_____ Set GCC-12 and G++-12 as default compiler _____"
export CC=/usr/bin/gcc-12
export CXX=/usr/bin/g++-12

cd ..
export CC=$(which gcc-12) &&\
export CXX=$(which g++-12) &&\
cargo run --color=always --release --features final -- --config /home/parity/authority.toml