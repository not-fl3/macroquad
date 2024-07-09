use macroquad::{
    file::load_file,
    gizmos::{draw_gizmos, gizmos_add_line, init_gizmos},
    math::{vec2, vec3},
    quad_gl::{camera::Environment, color},
    time::get_time,
    window::next_frame,
};

mod orbit_camera;

async fn game(ctx: macroquad::Context) {
    unsafe {
        macroquad::miniquad::gl::glEnable(macroquad::miniquad::gl::GL_TEXTURE_CUBE_MAP_SEAMLESS)
    };

    init_gizmos(&ctx);

    let mut scene = ctx.new_scene();

    let heightmap = load_file("examples/ferris.png").await.unwrap();
    let heightmap = quad_gl::image::decode(&heightmap).unwrap();

    let indices = vec![0u16, 1, 2, 0, 2, 3];

    let vertices = vec![
        vec3(-0.5, 0., -0.5),
        vec3(-0.5, 0., 0.5),
        vec3(0.5, 0., 0.5),
        vec3(0.5, 0., -0.5),
    ];

    for v in &vertices {
        gizmos_add_line(true, *v, *v + vec3(0.0, 0.1, 0.0));
    }

    let uvs = vec![vec2(0., 1.), vec2(0., 0.), vec2(1., 0.), vec2(1., 1.)];
    let normals = vec![vec3(0., 1., 0.); 4];
    let mesh = quad_gl::models::CpuMesh(vertices, uvs, normals, indices);
    let mesh = ctx.mesh(
        mesh,
        Some(ctx.new_texture_from_rgba8(
            heightmap.width as _,
            heightmap.height as _,
            &heightmap.data,
        )),
    );
    let _mesh = scene.add_model(&mesh);
    let mut orbit = orbit_camera::OrbitCamera::new();

    loop {
        let t = get_time();
        let mut p = vec3(t.sin() as f32, 0.0, t.cos() as f32);

        gizmos_add_line(true, p, p * 1.2);
        gizmos_add_line(false, vec3(0.0, 0.0, 0.0), p);

        ctx.clear_screen(color::WHITE);
        orbit.orbit(&ctx);
        scene.draw(&orbit.camera);
        draw_gizmos(&orbit.camera);
        next_frame().await
    }
}

fn main() {
    macroquad::start(Default::default(), |ctx| game(ctx));
}
