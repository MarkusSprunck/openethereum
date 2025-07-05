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

if [ ! -d "/opt/homebrew/opt/bzip2" ] || \
   [ ! -d "/opt/homebrew/opt/lz4" ] || \
   [ ! -d "/opt/homebrew/opt/zstd" ] || \
   [ ! -d "/opt/homebrew/Cellar/snappy/1.2.2" ] || \
   [ ! -d "/opt/homebrew/Cellar/rocksdb/10.2.1" ]; then
    echo "Error: Required libraries not found. Please install missing packages with brew."
    exit 1
fi

#strip ON
export RUSTFLAGS="-L native=/opt/homebrew/opt/bzip2/lib \
                  -L native=/opt/homebrew/opt/lz4/lib \
                  -L native=/opt/homebrew/opt/zstd/lib \
                  -L/opt/homebrew/Cellar/snappy/1.2.2/lib \
                  -L/opt/homebrew/Cellar/rocksdb/10.2.1/lib \
                  -Clink-arg=-lbz2 \
                  -Clink-arg=-llz4 \
                  -Clink-arg=-lzstd \
                  -Clink-arg=-lz \
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
