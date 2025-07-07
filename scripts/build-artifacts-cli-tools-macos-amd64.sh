#!/bin/bash

# Ensure that following packages have been installed:
#
# brew install bzip2 lz4 zstd snappy rocksdb

set -e # fail on any error
set -u # treat unset variables as error

cd ..

echo "_____ Post-processing binaries _____"
rm -rf .artifacts/*
mkdir -p .artifacts/




echo "_____ Set Rust Version _____"
rustup override set 1.85

echo "_____ Set GCC-12 and G++-12 as default compiler _____"
export CC="$(which gcc-12)"
export CXX="$(which g++-12)"

LIB_SNAPPY="$(brew --prefix snappy)"
LIB_ROCKSDB="$(brew --prefix rocksdb)"

echo "LIB_SNAPPY=$LIB_SNAPPY"
echo "LIB_ROCKSDB=$LIB_ROCKSDB"
echo "CC=$CC"
echo "CXX=$CXX"

if [ ! -d "$(brew --prefix snappy)" ] || \
   [ ! -d "$(brew --prefix rocksdb)" ]; then
    echo "Error: Required libraries not found. Please install missing packages with brew."
    exit 1
fi

#strip ON
export RUSTFLAGS="-L$(brew --prefix snappy)/lib \
                  -L$(brew --prefix rocksdb)/lib \
                  -Clink-arg=-s \
                  -Ctarget-feature=+aes"

echo "_____ Clean _____"
time cargo clean  -p ethstore-cli
time cargo clean  -p ethkey-cli

echo "_____ Build _____"
time cargo build --color=always --profile dev -p ethstore-cli
time cargo build --color=always --profile dev -p ethkey-cli

echo "_____ Clean copy of result files"
rm -rf .artifacts && mkdir  .artifacts
cp -v target/debug/ethkey   .artifacts/ethkey
cp -v target/debug/ethstore .artifacts/ethstore
