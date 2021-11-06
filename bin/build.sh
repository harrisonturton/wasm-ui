#!/bin/bash

# This script will generate a build/web directory (among other artefacts) that
# provides the functioning website when served from a webserver.

ROOT_PATH="$(dirname `which $0`)/../"
cd $ROOT_PATH

echo $1

if [[ $1 -eq core ]]; then
	wasm-pack build --out-dir ../build/core core
	exit 0
fi

if [[ $1 -eq web ]]; then
	npm run build --prefix web
	exit 0
fi

wasm-pack build --out-dir ../build/core core
npm run build --prefix web
