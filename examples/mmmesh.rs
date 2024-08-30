use macroquad::{
    file::load_file,
    gizmos::{draw_gizmos, gizmos_add_line, init_gizmos},
    math::{vec2, vec3, Vec2, Vec3},
    quad_gl::{color, scene::Shader},
    time::get_time,
    window::next_frame,
};

mod orbit_camera;

const vertices: &[Vec3] = &[
    vec3(-0.5, 0.5, 0.8),
    vec3(-0.5, 0.5, -0.8),
    vec3(-0.5, -0.5, 0.8),
    vec3(-0.5, 0.5, -0.8),
    vec3(-0.5, -0.5, -0.8),
    vec3(-0.5, -0.5, 0.8),
    vec3(-0.5, 0.5, -0.8),
    vec3(0.5, 0.5, -0.8),
    vec3(-0.5, -0.5, -0.8),
    vec3(0.5, 0.5, -0.8),
    vec3(0.5, -0.5, -0.8),
    vec3(-0.5, -0.5, -0.8),
    vec3(0.5, 0.5, -0.8),
    vec3(0.5, 0.5, 0.8),
    vec3(0.5, -0.5, 0.8),
    vec3(0.5, -0.5, -0.8),
    vec3(0.5, 0.5, -0.8),
    vec3(0.5, -0.5, 0.8),
    vec3(0.5, 0.5, 0.8),
    vec3(-0.5, 0.5, 0.8),
    vec3(-0.5, -0.5, 0.8),
    vec3(0.5, -0.5, 0.8),
    vec3(0.5, 0.5, 0.8),
    vec3(-0.5, -0.5, 0.8),
    vec3(-0.5, -0.5, 0.8),
    vec3(-0.5, -0.5, -0.8),
    vec3(0.5, -0.5, -0.8),
    vec3(0.5, -0.5, 0.8),
    vec3(-0.5, -0.5, 0.8),
    vec3(0.5, -0.5, -0.8),
    vec3(0.5, 0.5, 0.8),
    vec3(0.5, 0.5, -0.8),
    vec3(-0.5, 0.5, -0.8),
    vec3(-0.5, 0.5, 0.8),
    vec3(0.5, 0.5, 0.8),
    vec3(-0.5, 0.5, -0.8),
];
const uvs: &[Vec2] = &[
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 0.0),
];
const normals: &[Vec3] = &[
    vec3(-1.0, 0.0, 0.0),
    vec3(-1.0, 0.0, 0.0),
    vec3(-1.0, 0.0, 0.0),
    vec3(-1.0, 0.0, 0.0),
    vec3(-1.0, 0.0, 0.0),
    vec3(-1.0, 0.0, 0.0),
    vec3(0.0, 0.0, -1.0),
    vec3(0.0, 0.0, -1.0),
    vec3(0.0, 0.0, -1.0),
    vec3(0.0, 0.0, -1.0),
    vec3(0.0, 0.0, -1.0),
    vec3(0.0, 0.0, -1.0),
    vec3(1.0, -0.0, 0.0),
    vec3(1.0, -0.0, 0.0),
    vec3(1.0, -0.0, 0.0),
    vec3(1.0, 0.0, 0.0),
    vec3(1.0, 0.0, 0.0),
    vec3(1.0, 0.0, 0.0),
    vec3(0.0, 0.0, 1.0),
    vec3(0.0, 0.0, 1.0),
    vec3(0.0, 0.0, 1.0),
    vec3(0.0, 0.0, 1.0),
    vec3(0.0, 0.0, 1.0),
    vec3(0.0, 0.0, 1.0),
    vec3(0.0, -1.0, 0.0),
    vec3(0.0, -1.0, 0.0),
    vec3(0.0, -1.0, 0.0),
    vec3(0.0, -1.0, 0.0),
    vec3(0.0, -1.0, 0.0),
    vec3(0.0, -1.0, 0.0),
    vec3(0.0, 1.0, 0.0),
    vec3(0.0, 1.0, 0.0),
    vec3(0.0, 1.0, 0.0),
    vec3(0.0, 1.0, 0.0),
    vec3(0.0, 1.0, 0.0),
    vec3(0.0, 1.0, 0.0),
];
const indices: &[u16] = &[
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35,
];

async fn game(ctx: macroquad::Context) {
    unsafe {
        macroquad::miniquad::gl::glEnable(macroquad::miniquad::gl::GL_TEXTURE_CUBE_MAP_SEAMLESS)
    };

    init_gizmos(&ctx);

    let mut scene = ctx.new_scene();

    let texture = load_file("examples/ferris.png").await.unwrap();
    let texture = quad_gl::image::decode(&texture).unwrap();

    let mesh = quad_gl::models::CpuMesh(
        vertices.to_vec(),
        uvs.to_vec(),
        normals.to_vec(),
        indices.to_vec(),
    );
    let mut mesh = ctx.mesh(mesh, None);
    mesh.nodes[0].materials[0].shader = Shader::new(
        ctx.quad_ctx.lock().unwrap().as_mut(),
        vec![],
        Some(FRAGMENT),
        None,
    );

    let _mesh = scene.add_model(&mesh);
    let mut orbit = orbit_camera::OrbitCamera::new();

    loop {
        let t = get_time();
        let p = vec3(t.sin() as f32, 0.0, t.cos() as f32);

        gizmos_add_line(true, p, p * 1.2);
        gizmos_add_line(false, vec3(0.0, 0.0, 0.0), p);

        ctx.clear_screen(color::BLACK);
        orbit.orbit(&ctx);
        scene.draw(&orbit.camera);
        //draw_gizmos(&orbit.camera);
        next_frame().await
    }
}

fn main() {
    macroquad::start(Default::default(), |ctx| game(ctx));
}

const VERTEX: &str = r#"
#include "common_vertex.glsl"

void vertex() {
}
"#;

const FRAGMENT: &str = r#"
varying vec3 out_normal;

void main() {
    vec3 norm = normalize(out_normal);
    vec3 lightDir = normalize(vec3(1.0, -1.0, 0.5));
    float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = diff * vec3(1.0) + vec3(0.3);

    gl_FragColor = vec4(diffuse,1.0);
}
"#;
