#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

echo "_____ Switch to Clang _____"
export CC=/usr/bin/clang &&\
export CXX=/usr/bin/clang++ &&\
cargo test --all
