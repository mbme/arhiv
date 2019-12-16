#! /usr/bin/env bash

./build/dev-server.sh $@ &
./build/dev-web-app.sh &

wait
