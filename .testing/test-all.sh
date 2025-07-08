#!/bin/bash

set -e # fail on any error
set -u # treat unset variables as error

cd ..
echo "_____ Switch to CGG _____"
export CC=$(which gcc-12) &&\
export CXX=$(which g++-12) &&\
time cargo test --all
