use macroquad::{
    math::{vec2, Vec2, Vec3},
    quad_gl::{
        color::RED,
        ui::{
            hash,
            widgets::{self, Group},
        },
    },
    window::next_frame,
};

async fn game(ctx: macroquad::Context) {
    let mut canvas = ctx.new_canvas();

    loop {
        ctx.clear_screen(RED);
        canvas.clear();

        ctx.root_ui().label(None, "Hello");
        ctx.root_ui().label(None, "AAAA");
        widgets::Window::new(hash!(), vec2(400., 200.), vec2(320., 400.))
            .label("Shop")
            .titlebar(true)
            .ui(&mut *ctx.root_ui(), |ui| {
                for i in 0..30 {
                    Group::new(hash!("shop", i), vec2(300., 80.)).ui(ui, |ui| {
                        ui.label(vec2(10., 10.), &format!("Item N {}", i));
                        ui.label(vec2(260., 40.), "10/10");
                        ui.label(vec2(200., 58.), &format!("{} kr", 800));
                        if ui.button(vec2(260., 55.), "buy") {
                            println!("aa");
                        }
                    });
                }
            });

        ctx.root_ui().draw(&mut canvas);
        canvas.draw();

        next_frame().await
    }
}

fn main() {
    macroquad::start(Default::default(), |ctx| game(ctx));
}
