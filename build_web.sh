#!/bin/sh

cargo build --target wasm32-unknown-unknown --release

rm -rf static
mkdir -p static/assets

cp target/wasm32-unknown-unknown/release/yinsh.wasm static
cp src/www/index.html static
cp assets/* static/assets
