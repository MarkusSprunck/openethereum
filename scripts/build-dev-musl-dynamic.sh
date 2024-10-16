#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

export RUSTFLAGS="-C target-feature=-crt-static -l /usr/lib/x86_64-linux-musl"
cargo build --profile dev --target x86_64-unknown-linux-musl