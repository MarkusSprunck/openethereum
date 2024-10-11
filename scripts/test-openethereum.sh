#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

cargo test --package openethereum