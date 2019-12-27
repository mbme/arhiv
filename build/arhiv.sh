#! /usr/bin/env bash

set -e

./node_modules/.bin/tsc \
  -p ./tsconfig.json \
  --noEmitOnError

./build/node.sh tsdist/src/arhiv/bin $@
