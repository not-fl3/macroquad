use macroquad::color::*;

async fn game(ctx: macroquad::Context) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let _guard = rt.enter();
    for _ in 0..3 {
        ctx.clear_screen(WHITE);

        eprintln!("now for some tokio business");

        let file = tokio::fs::File::open("examples/ferris.png").await.unwrap();

        eprintln!("tokio file loaded");

        ctx.next_frame().await;
    }
}

fn main() {
    macroquad::start(Default::default(), |ctx| game(ctx));
}
