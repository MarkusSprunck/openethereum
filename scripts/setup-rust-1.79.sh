#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

rustup toolchain add 1.79 --profile minimal
rustup install 1.79
rustup override set 1.79