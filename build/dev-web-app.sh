#! /usr/bin/env bash

set -e

# create empty file to make rollup --watch work
mkdir -p tsdist/src/web-app
touch tsdist/src/web-app/index.js

# web app typescript into javascript
./node_modules/.bin/tsc \
  -p src/web-app/tsconfig.json \
  --noEmitOnError \
  --watch --preserveWatchOutput &

# --diagnostics --listEmittedFiles --listFiles \
# --traceResolution \

# web app bundle
./node_modules/.bin/rollup -c ./build/web-app.rollup.config.js --watch &

wait
