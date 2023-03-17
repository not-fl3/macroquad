use macroquad::prelude::*;

#[macroquad::main("3D")]
async fn main() {
    let model = Model::load_gltf("examples/ship.gltf").await.unwrap();

    loop {
        scene_graph().clear(Color::new(0.2, 0.2, 0.5, 1.0));

        let mut canvas = scene_graph().fullscreen_canvas();
        canvas.draw_text("WELCOME TO 3D WORLD", 10.0, 20.0, 30.0, BLACK);
        canvas.draw_text("TEXT BELOW!!!", 400.0, 400.0, 30.0, BLUE);
        canvas.draw_rectangle(300., 200., 100., 100., RED);
        scene_graph().draw_canvas(canvas);

        scene_graph().draw_model(
            &mut RenderState {
                camera: Camera::Camera3D {
                    position: vec3(-10., 7.5, 0.),
                    up: vec3(0., 1., 0.),
                    target: vec3(0., 0., 0.),
                    fovy: 45.,
                    projection: macroquad::camera::Projection::Perspective,
                    aspect: None,
                },
                ..Default::default()
            },
            &model,
            Mat4::IDENTITY,
        );

        let mut canvas = scene_graph().fullscreen_canvas();
        canvas.draw_rectangle(100., 350., 100., 100., GREEN);
        canvas.draw_text("TEXT ABOVE!!!", 400.0, 300.0, 30.0, YELLOW);
        scene_graph().draw_canvas(canvas);

        next_frame().await
    }
}
