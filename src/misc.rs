//! Miscellaneous functions that do not fit anywhere else in the code

use std::{
    process::Command,
    env::consts
}; 

/// Open URL with default system browser
///```ignore
/// loop {
///     clear_background(LIGHTGRAY);
///     if root_ui().button(Vec2::new(10.0, 10.0), "Click me") {
///         open_url("https://macroquad.rs/");
///     }
/// 
///     next_frame().await
/// 
/// }
///```
// TODO: Add open url for wasm and other platforms if possible
pub fn open_url(url: &str) {
    match consts::OS {
        "linux" => {
            Command::new("xdg-open").arg(url).spawn().unwrap();
        }
        "macos" => {
            Command::new("open").arg(url).spawn().unwrap();
        }
        "windows" => {
            Command::new("explorer").arg(url).spawn().unwrap();
        }
        _ => {}
    }
}
