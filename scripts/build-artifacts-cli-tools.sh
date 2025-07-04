#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

cd ..

echo "_____ Set GCC-12 and G++-12 as default compiler _____"
export CC="$(which gcc-12)"
export CXX="$(which g++-12)"

echo "_____ Set Rust Verions _____"
rustup override set 1.85

#strip ON
export RUSTFLAGS=" -Clink-arg=-s -Ctarget-feature=+aes,+ssse3"

echo "_____ Build tools _____"

time cargo build --color=always --profile dev -p ethstore-cli
time cargo build --color=always --profile dev -p ethkey-cli

echo "_____ Post-processing binaries _____"
rm -rf .artifacts/*
mkdir -p .artifacts/

cp -v target/debug/ethstore .artifacts/ethstore
cp -v target/debug/ethkey .artifacts/ethkey
