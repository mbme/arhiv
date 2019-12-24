#! /usr/bin/env bash

NODE_NO_WARNINGS=1 node \
                --experimental-modules \
                --experimental-loader ./build/loader.js \
                --es-module-specifier-resolution=node \
                $@
