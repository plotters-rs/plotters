#!/bin/bash
set -e
CONFIG=release
mkdir -p www/pkg
cargo build --target=wasm32-unknown-unknown --release
wasm-bindgen --out-dir www/pkg   --target web target/wasm32-unknown-unknown/${CONFIG}/wasm_demo.wasm
cd www
echo "Goto http://localhost:8000/ to see the result"
python -m SimpleHTTPServer
