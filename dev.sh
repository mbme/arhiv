#! /usr/bin/env bash

# cleanup
rm -rf ./tsdist ./dist

# create empty file to make rollup --watch work
mkdir -p tsdist/web-app
touch tsdist/web-app/index.js

# ensure local arhiv dir exists
mkdir -p temp-arhiv-root

# server
NODE_NO_WARNINGS=1 LOG=DEBUG ./vnode src/arhiv/server/bin 8080 pass ./temp-arhiv-root --gen-data &

# web app typescript into javascript
./node_modules/.bin/tsc \
  -p src/web-app/tsconfig.json \
  --noEmitOnError \
  --module commonjs \
  --outDir ./tsdist \
  --watch --preserveWatchOutput &

# --diagnostics --listEmittedFiles --listFiles \
# --traceResolution \

# web app bundle
./node_modules/.bin/rollup -c web-app.rollup.config.js --watch &

wait
