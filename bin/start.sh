#!/bin/bash

# This script starts a webserver with a debug build of the site. If there is no
# build/core directory, it will build the core WASM libraries first. 

ROOT_PATH="$(dirname `which $0`)/../"
cd $ROOT_PATH

if [ ! -d "build/core" ]
then
	wasm-pack build --out-dir ../build/core core
fi

cd web
npm run start
