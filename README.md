# macroquad

[![Github Actions](https://github.com/not-fl3/macroquad/workflows/Cross-compile/badge.svg)](https://github.com/not-fl3/macroquad/actions?query=workflow%3A)
[![Crates.io version](https://img.shields.io/crates/v/macroquad.svg)](https://crates.io/crates/macroquad)
[![Discord chat](https://img.shields.io/discord/710177966440579103.svg?label=discord%20chat)](https://discord.gg/WfEp6ut)

`macroquad` is a simple and easy to use game library for Rust programming language, heavily inspired by [raylib](https://github.com/raysan5/raylib). 

`macroquad` attempts to avoid any rust-specific programming concepts like lifetimes/borrowing, making it very friendly for rust beginners.

## Supported platforms

* PC: Windows/Linux/MacOs
* HTML5
* Android
* IOS

## Features

* Same code for all supported platforms, no platform dependent defines required 
* Efficient 2D rendering with automatic geometry batching
* Minimal amount of dependencies: build after `cargo clean` takes only 16s on x230(~6years old laptop)
* Immediate mode UI library included
* Single command deploy for both WASM and Android [build instructions](https://github.com/not-fl3/miniquad/#building-examples)

<details>
<summary>tips</summary>
Adding the following snippet to your Cargo.toml ensures that all dependencies compile in release even in debug mode. In macroquad, this has the effect of making images load several times faster and your applications much more performant, while keeping compile times miraculously low.

```toml
[profile.dev.package.'*']
opt-level = 3
```
</details>

## async/await

While macroquad attempts to use as mini rust-specific concepts as possible, `.await` in every examples looks a bit scary.
Rust's `async/await` is used to solve just one problem - cross platform main loop ogranisation. 

<details>
<summary>details</summary>
  

The problem: on WASM and android its not really easy to organize main loop like this: 
```
fn main() {
    // do some initilization
    
    // start main loop
    loop {
        // handle input 

        // update logic

        // draw frame
    }
}
```

It is fixable on Android with threads, but on web there is not way to "pause" and "resume" wasm execution, so no wasm code should block ever.
While that loop is blocking for the entire game execution!
The C++ solution for that problem: https://kripken.github.io/blog/wasm/2019/07/16/asyncify.html

But in Rust we have async/await. Rust's `futures` is basicly a continuations - `future`'s stack may be store into a variable to later pause/resume execution of future's code.

async/await in macroquad is used without any external dependencies - no runtime, executor or even futures-rs are involved. Its just a way to preserve `main`'s stack on WASM and keep the code cross platform without any wasm-specific main loop.
</details>

# Platinum sponsors

Macroquad is supported by:

<p>
  <a href="https://embark-studios.com">
    <img src="https://www.embark.dev/img/logo_black.png" width="201px">
  </a>
</p>


