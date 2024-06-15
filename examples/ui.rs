use macroquad::{
    hash,
    prelude::*,
    ui::widgets::{self, Group},
};

async fn game(ctx: macroquad::Context3) {
    let mut canvas = ctx.new_canvas();
    let mut ui = ctx.new_ui();

    loop {
        canvas.clear(GRAY);

        ui.grab_input();
        ui.label(None, "Hello");
        ui.label(None, "AAAA");
        widgets::Window::new(hash!(), vec2(400., 200.), vec2(320., 400.))
            .label("Shop")
            .titlebar(true)
            .ui(&mut ui, |ui| {
                for i in 0..30 {
                    Group::new(hash!("shop", i), Vec2::new(300., 80.)).ui(ui, |ui| {
                        ui.label(Vec2::new(10., 10.), &format!("Item N {}", i));
                        ui.label(Vec2::new(260., 40.), "10/10");
                        ui.label(Vec2::new(200., 58.), &format!("{} kr", 800));
                        if ui.button(Vec2::new(260., 55.), "buy") {
                            //data.inventory.push(format!("Item {}", i));
                        }
                    });
                }
            });

        ui.draw(&mut canvas);
        canvas.draw();
        next_frame().await
    }
}

fn main() {
    macroquad::start(Default::default(), |ctx| game(ctx));
}
