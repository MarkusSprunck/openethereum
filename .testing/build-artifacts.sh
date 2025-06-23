#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

cd ..

echo "_____ Set GCC-12 and G++-12 as default compiler _____"
export CC=/usr/bin/gcc-12
export CXX=/usr/bin/g++-12

#strip ON
export RUSTFLAGS=" -Clink-arg=-s -Ctarget-feature=+aes"

echo "_____ Build tools _____"

time cargo build --color=always --profile dev -p ethstore-cli
time cargo build --color=always --profile dev -p ethkey-cli

echo "_____ Post-processing binaries _____"
rm -rf .artifacts/*
mkdir -p .artifacts/

cp -v target/debug/ethstore .artifacts/ethstore
cp -v target/debug/ethkey .artifacts/ethkey
