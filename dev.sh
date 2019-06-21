#! /usr/bin/env bash

# cleanup
rm -rf ./tsdist ./dist

# create empty file to make rollup --watch work
mkdir -p tsdist/web-app
touch tsdist/web-app/index.js

# server
NODE_NO_WARNINGS=1 LOG=DEBUG ./vnode src/isodb-server/bin 8080 pass /tmp/db --gen-data &

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
./node_modules/.bin/rollup -c --watch &

wait
