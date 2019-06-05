if not exist "www\pkg" mkdir www\pkg
rustup target add wasm32-unknown-unknown
cargo build --target=wasm32-unknown-unknown --release
wasm-bindgen --version
if errorlevel 1 cargo install wasm-bindgen-cli
wasm-bindgen --out-dir www\pkg --target web target\wasm32-unknown-unknown\release\wasm_demo.wasm
cd www
echo "Goto http://localhost:8000/ to see the demo!"
python ..\server.py
