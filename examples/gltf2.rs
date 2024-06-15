use macroquad::{prelude::*, sprite_layer::SpriteLayer};

async fn game(ctx: macroquad::Context3) {
    let mut scene = ctx.new_scene();

    let helmet = ctx.load_gltf("examples/DamagedHelmet.gltf").await.unwrap();
    //let helmet = ctx.load_gltf("examples/tree.gltf").await.unwrap();
    let _helmet = scene.add_model(&helmet);

    let plane = scene.mesh(macroquad::models::square());
    let plane = scene.add_model(&plane);

    {
        let plane = scene.borrow_model(&plane);
        plane.translation = vec3(0., -2.0, 0.);
        plane.scale = vec3(10.0, 1.0, 10.);
    }

    let skybox = ctx
        .load_cubemap(
            "examples/skybox/skybox_px.png",
            "examples/skybox/skybox_nx.png",
            "examples/skybox/skybox_py.png",
            "examples/skybox/skybox_ny.png",
            "examples/skybox/skybox_pz.png",
            "examples/skybox/skybox_nz.png",
        )
        .await
        .unwrap();

    let mut camera = Camera {
        environment: Environment::Skybox(skybox),
        depth_enabled: true,
        projection: Projection::Orthographic,
        position: vec3(0., 1.5, 4.),
        up: vec3(0., 1., 0.),
        target: vec3(0., 0., 0.),
        z_near: 0.1,
        z_far: 15.0,
        ..Default::default()
    };

    let mut canvas = ctx.new_sprite_layer();
    canvas.draw_text("WELCOME TO 3D WORLD", 10.0, 20.0, 30.0, BLACK);
    ShapeBuilder::rectangle(vec2(100., 100.), RED)
        .position(vec2(100., 100.))
        .draw(&mut canvas);

    let mut zoom = 1.0;
    loop {
        if is_mouse_button_down(MouseButton::Left) {}
        if mouse_wheel().1 != 0.0 {
            zoom -= mouse_wheel().1 * 0.1;
        }
        {
            camera.position = vec3(0.0, 0.0, 3.0) + vec3(0.0, 1.5, 5.0) * zoom;
            camera.up = vec3(0.0, 1.0, 0.0);
            camera.target = vec3(0.0, 0.0, 3.0);
        }

        scene.draw(&camera);
        canvas.draw();
        next_frame().await
    }
}

fn main() {
    macroquad::start(Default::default(), |ctx| game(ctx));
}
