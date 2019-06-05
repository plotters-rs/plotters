#!/bin/bash
set -e
CONFIG=release
mkdir -p www/pkg
rustup target add wasm32-unknown-unknown
if [ -z "$(cargo install --list | grep wasm-bindgen-cli)" ]
then
	cargo install wasm-bindgen-cli
fi
cargo build --target=wasm32-unknown-unknown --release
wasm-bindgen --out-dir www/pkg   --target web target/wasm32-unknown-unknown/${CONFIG}/wasm_demo.wasm
cd www
echo "Goto http://localhost:8000/ to see the result"
python ../server.py
