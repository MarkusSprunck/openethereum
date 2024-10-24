#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

cd ..
cargo run --color=always --release --features final -- --config /home/parity/authority.toml