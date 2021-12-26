use macroquad::prelude::*;

#[macroquad::main("Window")]
async fn main() {
    let mut c: Clock = Clock::new(); // Make a clock

    loop {
        clear_background(WHITE);
        c.tick(); // Update the clock

        if is_key_pressed(KeyCode::P) {
            c.pause(); // Stop the clock
            println!("Clock paused");
        }
        else if is_key_pressed(KeyCode::R) {
            c.resume(); // Resume the clock
            println!("Clock resumed")
        }

        if is_key_pressed(KeyCode::Space) {
            println!("Current time: {}", c.get_elpased_time());
        }

        if c.get_elpased_time() > 10.0 { // Check is the current elapsed time is gretaer than 100 seconds
            c.restart(); // Restart clock
            println!("Clock restarted at 10 seconds.")
        }
            
        next_frame().await
    }
}