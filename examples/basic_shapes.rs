use macroquad::prelude::*;

async fn game(ctx: macroquad::Context3) {
    let mut canvas = ctx.new_canvas();
    loop {
        canvas.clear(WHITE);
        ShapeBuilder::line(vec2(40.0, 40.0), vec2(100.0, 200.0))
            .color(BLUE)
            .draw(&mut canvas);
        ShapeBuilder::rectangle(vec2(120.0, 60.0))
            .position(vec2(ctx.screen_width() / 2.0 - 60.0, 100.0))
            .color(GREEN)
            .draw(&mut canvas);
        ShapeBuilder::circle(15.0)
            .position(vec2(ctx.screen_width() - 30.0, ctx.screen_height() - 30.0))
            .color(YELLOW)
            .draw(&mut canvas);
        ShapeBuilder::new(Text::new("HELLO").font_size(30.0))
            .position(20.0, 20.0)
            .color(DARKGRAY)
            .draw(&mut canvas);
        ctx.draw_canvas(&mut canvas);
        next_frame().await
    }
}

fn main() {
    macroquad::start(Default::default(), |ctx| game(ctx));
}
