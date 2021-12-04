use macroquad::color::{BLACK, WHITE};
use macroquad::experimental::collections::storage;
use macroquad::experimental::coroutines::start_coroutine;
use macroquad::file::{FileError, load_string};
use macroquad::math::{Rect};
use macroquad::time::get_time;
use macroquad::text::{draw_text};
use macroquad::window::{clear_background, next_frame, screen_height, screen_width};
use macroquad::input::{is_key_down, KeyCode};
use macroquad::texture::{load_texture};
use macroquad_tiled::{Error, load_tileset, TileSet};


// impl From<FileError> for macroquad_tiled::Error {
//     fn from(e: FileError) -> Self {
//         todo!()
//     }
// }

fn from_file_error_to_error(e: FileError) -> macroquad_tiled::Error {
    macroquad_tiled::Error::TextureNotFound {
        texture: e.path
    }
}

struct Resources {
    sheet: TileSet,
}

impl Resources {
    pub async fn new() -> Result<Self, Error> {
        let json = load_string("resources/mountain_landscape.json")
            .await
            .map_err(from_file_error_to_error)
            ?;

        let sheet = load_texture("resources/mountain_landscape.png")
            .await
            .map_err(from_file_error_to_error)
            ?;

        let sheet = load_tileset(&json, sheet)?;

        Ok(Self {
            sheet
        })
    }
}

#[macroquad::main("TileTest")]
async fn main() {

    let resources_loading = start_coroutine(async move {
        let resources = Resources::new().await.expect("Could not load resources");
        storage::store(resources);
    });

    while ! resources_loading.is_done() {
        clear_background(BLACK);
        draw_text(
            &format!(
                "Loading resources {}",
                ".".repeat(((get_time() * 2.0) as usize) % 4)
            ),
            screen_width() / 2.0 - 160.0,
            screen_height() / 2.0,
            40.,
            WHITE,
        );

        next_frame().await
    }

    let resources = storage::get::<Resources>();

    loop {

        if is_key_down(KeyCode::Q) {
            return;
        }

        clear_background(WHITE);

        for x in 0..10 {
            for y in 0..10 {
                resources.sheet.spr(
                    x + y * 16,
                    Rect {
                        x: 34.0 * x as f32,
                        y: 34.0 * y as f32,
                        w: 32.0,
                        h: 32.0
                    }
                );
            }
        }

        next_frame().await
    }
}
