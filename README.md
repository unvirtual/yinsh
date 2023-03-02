# YINSH

This is an implementation of the abstract board game YINSH in Rust using [macroquad](https://github.com/not-fl3/macroquad).

You can play it [in the browser](https://unvirtual.github.io/yinsh)!

![screenshot](https://raw.githubusercontent.com/unvirtual/yinsh-rs/main/screenshot/screenshot.png)

## Building for the web

Install the required wasm build target and run the provided build script

    rustup target add wasm32-unknown-unknown
    ./build_web.sh

This creates a directory `./static` that can be served e.g. with `basic-http-server`
  
    cargo install basic-http-server
    basic-http-server static

### External assets

Included fonts are all relased under the OFL, see OFL.txt.