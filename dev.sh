#! /usr/bin/env bash

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
