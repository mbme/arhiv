#! /usr/bin/env bash

export BASE_DIR=tsdist/web-app-src

# cleanup
rm -rf $BASE_DIR

# create empty file to make rollup --watch work
mkdir -p $BASE_DIR/web-app
touch $BASE_DIR/web-app/index.js
touch $BASE_DIR/web-app/serviceWorker.js

# web app typescript into javascript
./node_modules/.bin/tsc \
  -p src/web-app/tsconfig.json \
  --noEmitOnError \
  --outDir $BASE_DIR \
  --watch --preserveWatchOutput &

# --diagnostics --listEmittedFiles --listFiles \
# --traceResolution \

# web app bundle
./node_modules/.bin/rollup -c ./build/web-app.rollup.config.js --watch &

wait
