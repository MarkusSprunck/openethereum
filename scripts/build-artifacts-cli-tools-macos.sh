#!/bin/bash

# Ensure that following packages have been installed:
#
# brew install bzip2
# brew install lz4
# brew install zstd
# brew install snappy
# brew install rocksdb

set -e # fail on any error
set -u # treat unset variables as error

cd ..

echo "_____ Post-processing binaries _____"
rm -rf .artifacts/*
mkdir -p .artifacts/

echo "_____ Set Rust Verions _____"
rustup override set 1.85

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

echo "_____ Build tools _____"
time cargo build --color=always --profile dev -p ethstore-cli
time cargo build --color=always --profile dev -p ethkey-cli

cp -v target/debug/ethkey .artifacts/ethkey
cp -v target/debug/ethstore .artifacts/ethstore
