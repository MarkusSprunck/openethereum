#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error
#strip ON
export RUSTFLAGS=" -Clink-arg=-s -Ctarget-feature=+aes"

echo "_____ Build OpenEthereum and tools _____"

time cargo build --color=always --profile dev --features final
time cargo build --color=always --profile dev -p evmbin
time cargo build --color=always --profile dev -p ethstore-cli
time cargo build --color=always --profile dev -p ethkey-cli

echo "_____ Post-processing binaries _____"
rm -rf .artifacts/*
mkdir -p .artifacts/

cp -v target/debug/openethereum .artifacts/openethereum
cp -v target/debug/openethereum-evm .artifacts/openethereum-evm
cp -v target/debug/ethstore .artifacts/ethstore
cp -v target/debug/ethkey .artifacts/ethkey
