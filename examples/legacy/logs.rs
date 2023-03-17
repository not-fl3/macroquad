//! Macroquad expose all the logging macros.
//! They will use browser console, android console or just stdout depending on the platform.
//! Those macros are the recommended way to output debug traces and logs.

use macroquad::prelude::*;

#[macroquad::main("Logs")]
async fn main() {
    debug!("This is a debug message");
    info!("and info message");
    error!("and errors, the red ones!");
    warn!("Or warnings, the yellow ones.");

    loop {
        clear_background(LIGHTGRAY);

        debug!("Still alive!");

        next_frame().await
    }
}
