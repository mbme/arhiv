#! /usr/bin/env bash

export BASE_DIR=tsdist/tester-src

# cleanup
rm -rf $BASE_DIR

# compile app
./node_modules/.bin/tsc \
  -p ./tsconfig.json \
  --noEmitOnError \
  --outDir $BASE_DIR


# run tests
./build/node.sh $BASE_DIR/tester/bin $@
