#! /usr/bin/env bash

export BASE_DIR=tsdist/arhiv-src

# cleanup
#rm -rf $BASE_DIR

./node_modules/.bin/tsc \
  -p ./tsconfig.json \
  --noEmitOnError \
  --outDir $BASE_DIR

./build/node.sh $BASE_DIR/arhiv/bin $@
