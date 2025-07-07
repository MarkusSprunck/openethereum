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

LIB_BZIP2="$(brew --prefix bzip2)"
LIB_LZ4="$(brew --prefix lz4)"
LIB_ZSTD="$(brew --prefix zstd)"
LIB_SNAPPY="$(brew --prefix snappy)"
LIB_ROCKSDB="$(brew --prefix rocksdb)"

echo "LIB_BZIP2=$LIB_BZIP2"
echo "LIB_LZ4=$LIB_LZ4"
echo "LIB_ZSTD=$LIB_ZSTD"
echo "LIB_SNAPPY=$LIB_SNAPPY"
echo "LIB_ROCKSDB=$LIB_ROCKSDB"
echo "CC=$CC"
echo "CXX=$CXX"

if [ ! -d "$LIB_BZIP2" ]  || \
   [ ! -d "$LIB_LZ4" ]    || \
   [ ! -d "$LIB_ZSTD" ]   || \
   [ ! -d "$LIB_SNAPPY" ] || \
   [ ! -d "$LIB_ROCKSDB" ]; then
    echo "Error: Required libraries not found. Please install missing packages with brew."
    exit 1
fi

#strip ON
export RUSTFLAGS="-L$(brew --prefix bzip2)/lib \
                  -L$(brew --prefix lz4)/lib \
                  -L$(brew --prefix zstd)/lib \
                  -L$(brew --prefix snappy)/lib \
                  -L$(brew --prefix rocksdb)/lib \
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
