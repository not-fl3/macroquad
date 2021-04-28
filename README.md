# macroquad

[![Github Actions](https://github.com/not-fl3/macroquad/workflows/Cross-compile/badge.svg)](https://github.com/not-fl3/macroquad/actions?query=workflow%3A)
[![Docs](https://docs.rs/macroquad/badge.svg?version=0.3.0-alpha)](https://docs.rs/macroquad/0.3.0-alpha/macroquad/index.html)
[![Crates.io version](https://img.shields.io/crates/v/macroquad.svg)](https://crates.io/crates/macroquad)
[![Discord chat](https://img.shields.io/discord/710177966440579103.svg?label=discord%20chat)](https://discord.gg/WfEp6ut)

`macroquad` is a simple and easy to use game library for Rust programming language, heavily inspired by [raylib](https://github.com/raysan5/raylib).

`macroquad` attempts to avoid any Rust-specific programming concepts like lifetimes/borrowing, making it very friendly for Rust beginners. See the [docs](https://docs.rs/macroquad/0.3.0-alpha/macroquad/index.html).

## Features

* Same code for all supported platforms, no platform dependent defines required
* Efficient 2D rendering with automatic geometry batching
* Minimal amount of dependencies: build after `cargo clean` takes only 16s on x230(~6years old laptop)
* Immediate mode UI library included
* Single command deploy for both WASM and Android [build instructions](https://github.com/not-fl3/miniquad/#building-examples)

## Supported platforms

* PC: Windows/Linux/MacOs
* HTML5
* Android
* IOS

## Build instructions

### Setting up a macroquad project

Macroquad is a normal rust dependency, therefore an empty macroquad project may be created with:

```bash
# Create empty cargo project
cargo init --bin
```

Add macroquad as a dependency to Cargo.toml:
```toml

[dependencies]
macroquad = "0.3"
```

Put some macroquad code in `src/main.rs`:
```rust
use macroquad::prelude::*;

#[macroquad::main("BasicShapes")]
async fn main() {
    loop {
        clear_background(RED);

        draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
        draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, YELLOW);

        draw_text("IT WORKS!", 20.0, 20.0, 30.0, DARKGRAY);

        next_frame().await
    }
}
```

And to run it natively: 
```bash
cargo run
```

For more examples take a look on [Macroquad examples folder](https://github.com/not-fl3/macroquad/tree/master/examples)

### linux

```bash
# ubuntu system dependencies
apt install libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev
```

### windows

On windows both MSVC and GNU target are supported, no additional dependencies required. 

Also cross-compilation to windows from linux is supported:

```sh
rustup target add x86_64-pc-windows-gnu

cargo run --target x86_64-pc-windows-gnu
```

### wasm

```sh
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown
```

This will produce .wasm file in `target/debug/wasm32-unknown-unknown/CRATENAME.wasm` or in `target/release/wasm32-unknown-unknown/CRATENAME.wasm` if built with `--release`. 

And then use the following .html to load it:

<details><summary>index.html</summary>

```html
<html lang="en">

<head>
    <meta charset="utf-8">
    <title>TITLE</title>
    <style>
        html,
        body,
        canvas {
            margin: 0px;
            padding: 0px;
            width: 100%;
            height: 100%;
            overflow: hidden;
            position: absolute;
            background: black;
            z-index: 0;
        }
    </style>
</head>

<body>
    <canvas id="glcanvas" tabindex='1'></canvas>
    <!-- Minified and statically hosted version of https://github.com/not-fl3/macroquad/blob/master/js/mq_js_bundle.js -->
    <script src="https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js"></script>
    <script>load("CRATENAME.wasm");</script> <!-- Your compiled wasm file -->
</body>

</html>
```
</details>

One of the ways to server static .wasm and .html:

```sh
cargo install basic-http-server
basic-http-server .
```

<details>
<summary>tips</summary>
Adding the following snippet to your Cargo.toml ensures that all dependencies compile in release even in debug mode. In macroquad, this has the effect of making images load several times faster and your applications much more performant, while keeping compile times miraculously low.

```toml
[profile.dev.package.'*']
opt-level = 3
```
</details>

## async/await

While macroquad attempts to use as few Rust-specific concepts as possible, `.await` in all examples looks a bit scary.
Rust's `async/await` is used to solve just one problem - cross platform main loop organization.

<details>
<summary>details</summary>


The problem: on WASM and android it's not really easy to organize the main loop like this:
```
fn main() {
    // do some initialization

    // start main loop
    loop {
        // handle input

        // update logic

        // draw frame
    }
}
```

It is fixable on Android with threads, but on web there is not way to "pause" and "resume" WASM execution, so no WASM code should block ever.
While that loop is blocking for the entire game execution!
The C++ solution for that problem: https://kripken.github.io/blog/wasm/2019/07/16/asyncify.html

But in Rust we have async/await. Rust's `futures` is basically a continuations - `future`'s stack may be store into a variable to later pause/resume execution of future's code.

async/await in macroquad is used without any external dependencies - no runtime, executor or even futures-rs are involved. It's just a way to preserve `main`'s stack on WASM and keep the code cross platform without any WASM-specific main loop.
</details>

## Community

- [Quads Discord server](https://discord.gg/WfEp6ut) - a place to chat with the library's devs and other community members.
- [Awesome Quads](https://github.com/ozkriff/awesome-quads) - a curated list of links to miniquad/macroquad-related code & resources.

# Platinum sponsors

Macroquad is supported by:

<p>
  <a href="https://embark-studios.com">
    <img src="https://www.embark.dev/img/logo_black.png" width="201px">
  </a>
</p>
