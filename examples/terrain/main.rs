use macroquad::{
    file::load_file,
    gizmos::{draw_gizmos, gizmos_add_line, init_gizmos},
    math::{vec2, vec3},
    quad_gl::{camera::Environment, color},
    window::next_frame,
};

mod orbit_camera;

async fn game(ctx: macroquad::Context) {
    unsafe {
        macroquad::miniquad::gl::glEnable(macroquad::miniquad::gl::GL_TEXTURE_CUBE_MAP_SEAMLESS)
    };

    init_gizmos(&ctx);

    let mut scene = ctx.new_scene();

    let heightmap = load_file("examples/heightmap.png").await.unwrap();
    let heightmap = quad_gl::image::decode(&heightmap).unwrap();

    let mut vertices = vec![];
    let mut uvs = vec![];
    let mut normals = vec![];
    let mut indices = vec![];

    let n = 200usize;
    let h = |x, y| {
        let x = x as f32 / n as f32;
        let y = y as f32 / n as f32;
        let ix = (x * heightmap.width as f32) as usize;
        let iy = (y * heightmap.height as f32) as usize;
        let ix = (ix + iy * heightmap.width) * 4;
        vec3(
            (x - 0.5) * 20.0,
            heightmap.data[ix] as f32 / 255.0 * 1.5,
            (y - 0.5) * 20.0,
        )
    };
    for j in 0..n {
        for i in 0..n {
            let v = h(i, j);
            vertices.push(v);
            if i == n - 1 || j == n - 1 {
                normals.push(vec3(0.0, 1.0, 0.0));
            } else {
                let v1 = h(i + 1, j);
                let v2 = h(i, j + 1);
                let n = -(v1 - v).normalize().cross((v2 - v).normalize());
                normals.push(n);
                gizmos_add_line(true, v, v + n * 0.1);
            }
            let x = i as f32 / n as f32;
            let y = j as f32 / n as f32;

            uvs.push(vec2(x, y));
        }
    }
    let mut i = 0;
    for _ in 0..n - 1 {
        for _ in 0..n - 1 {
            indices.push(i);
            indices.push(i + 1);
            indices.push(i + n as u16);
            indices.push(i + 1);
            indices.push(i + n as u16);
            indices.push(i + n as u16 + 1);
            i += 1;
        }
        i += 1;
    }

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

    let skybox = ctx
        .resources
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

    let mut orbit = orbit_camera::OrbitCamera::new();
    //orbit.camera.environment = Environment::Skybox(skybox);

    loop {
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
