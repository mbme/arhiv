#! /usr/bin/env bash

export BASE_DIR=tsdist/arhiv-server-src

# cleanup
rm -rf $BASE_DIR

# ensure local arhiv dir exists
mkdir -p temp-arhiv-root

# compile server
./node_modules/.bin/tsc \
  -p ./tsconfig.json \
  --noEmitOnError \
  --outDir $BASE_DIR

# run server
LOG=DEBUG ./node.sh $BASE_DIR/arhiv/server/bin 8080 pass ./temp-arhiv-root --gen-data
