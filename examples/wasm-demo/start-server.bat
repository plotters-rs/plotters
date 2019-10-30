if not exist "www\pkg" mkdir www\pkg
rustup target add wasm32-unknown-unknown
wasm-pack build --release
if errorlevel 1 cargo install wasm-pack
wasm-pack build --release
cd www
npm install
npm start
