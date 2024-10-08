#!/usr/bin/env bash

echo "###################################################################################"
echo "# 1. Create Secrets and Configuration"
echo "###################################################################################"


rm -rf dist/staging
./genKeys.sh -x staging

