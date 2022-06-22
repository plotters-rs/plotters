# The Minimal Demo for Plotters + WASM

To build the demo you need [wasm-pack](https://rustwasm.github.io/docs/book/game-of-life/setup.html).

Then you can run it locally either using `npm` and `webpack-dev-server` or
just with static web server.

The following script will install needed software and run the server via `npm`.
```
./start-server.sh
```

For Windows users without Bash, `start-server.bat` can be used to
launch the server.

```
start-server.bat
```

## Developing with NPM
Please use [rust-wasm guide](https://rustwasm.github.io/docs/book/game-of-life/setup.html) for initial setup .
Then you can run the demo locally using `npm`:
```bash
wasm-pack build
cd www
npm install
npm start
```

This will start a dev server which will automatically reload your page
whenever you change anything in `www` directory. To update `rust` code
call `wasm-pack build` manually.

## Developing without dependenices
If you don't want to use `npm` here's how you can run the example
using any web server. We are using rust [basic-http-server](https://github.com/brson/basic-http-server), but
any web server will do.

```bash
# Install web server (instead you can use your local nginx for example)
cargo install basic-http-server
wasm-pack build --target web # Note `--target web`
basic-http-server
```

Then open http://127.0.0.1:4000/www
