#! /usr/bin/env bash

set -e

# compile app
./node_modules/.bin/tsc \
  -p ./tsconfig.json \
  --noEmitOnError

# run tests
./build/node.sh tsdist/src/tester/bin $@
