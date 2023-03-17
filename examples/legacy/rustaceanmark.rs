use macroquad::prelude::*;

struct Rustaceane {
    pos: Vec2,
    speed: Vec2,
    color: Color,
}

#[macroquad::main("Rustaceanmark")]
async fn main() {
    let mut rustaceanes: Vec<Rustaceane> = Vec::new();
    let rustacean_tex = load_texture("examples/rustacean_happy.png").await.unwrap();
    rustacean_tex.set_filter(FilterMode::Nearest);

    loop {
        clear_background(Color::default());

        if macroquad::input::is_mouse_button_down(MouseButton::Left) {
            for _i in 0..100 {
                rustaceanes.push(Rustaceane {
                    pos: Vec2::from(macroquad::input::mouse_position()),
                    speed: Vec2::new(
                        rand::gen_range(-250., 250.) / 60.,
                        rand::gen_range(-250., 250.) / 60.,
                    ),
                    color: Color::from_rgba(
                        rand::gen_range(50, 240),
                        rand::gen_range(80, 240),
                        rand::gen_range(100, 240),
                        255,
                    ),
                })
            }
        }

        for rustaceane in &mut rustaceanes {
            rustaceane.pos += rustaceane.speed;

            if ((rustaceane.pos.x + rustacean_tex.width() / 2.) > screen_width())
                || ((rustaceane.pos.x + rustacean_tex.width() / 2.) < 0.)
            {
                rustaceane.speed.x *= -1.;
            }
            if ((rustaceane.pos.y + rustacean_tex.height() / 2.) > screen_height())
                || ((rustaceane.pos.y + rustacean_tex.height() / 2.) < 0.)
            {
                rustaceane.speed.y *= -1.;
            }

            draw_texture(
                &rustacean_tex,
                rustaceane.pos.x,
                rustaceane.pos.y,
                rustaceane.color,
            );
        }

        draw_text(format!("FPS: {}", get_fps()).as_str(), 0., 16., 32., WHITE);
        draw_text(
            format!("Rustaceanes: {}", rustaceanes.len()).as_str(),
            0.,
            32.,
            32.,
            WHITE,
        );

        next_frame().await
    }
}
