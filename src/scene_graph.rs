use crate::{
    camera::RenderState,
    color::Color,
    error::Error,
    file::load_file,
    get_context,
    material::Material,
    math::{vec2, vec3, Mat4},
    window::miniquad::*,
};

pub struct Model {
    bindings: Vec<Bindings>,
}

impl Model {
    pub async fn load_gltf(path: &str) -> Result<Model, Error> {
        let bytes = load_file(path).await?;

        let (gltf, buffers, _images) = gltf::import_slice(&bytes).unwrap();
        assert!(gltf.meshes().len() == 1);

        let mesh = gltf.meshes().next().unwrap();

        let mut bindings = Vec::new();

        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let indices: Vec<u16> = reader
                .read_indices()
                .unwrap()
                .into_u32()
                .map(|ix| ix as u16)
                .collect::<Vec<_>>();
            let vertices: Vec<[f32; 3]> = reader.read_positions().unwrap().collect::<Vec<_>>();
            let uvs: Vec<[f32; 2]> = if let Some(reader) = reader.read_tex_coords(0) {
                reader.into_f32().collect::<Vec<_>>()
            } else {
                vec![]
            };

            let normals: Vec<[f32; 3]> = reader.read_normals().unwrap().collect::<Vec<_>>();

            //println!("{:#?}", vertices);

            let ctx = &mut get_context().quad_context;
            let white_texture = ctx.new_texture_from_rgba8(1, 1, &[255, 255, 255, 255]);
            let vertex_buffer = ctx.new_buffer(
                BufferType::VertexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(&vertices),
            );
            let normals_buffer = ctx.new_buffer(
                BufferType::VertexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(&normals),
            );
            let uvs_buffer = ctx.new_buffer(
                BufferType::VertexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(&uvs),
            );
            let index_buffer = ctx.new_buffer(
                BufferType::IndexBuffer,
                BufferUsage::Immutable,
                BufferSource::slice(&indices),
            );
            bindings.push(Bindings {
                vertex_buffers: vec![vertex_buffer, uvs_buffer, normals_buffer],
                index_buffer,
                images: vec![white_texture, white_texture],
            });
        }

        Ok(Model { bindings })
    }
}

pub fn square() -> Model {
    let ctx = &mut get_context().quad_context;
    let white_texture = ctx.new_texture_from_rgba8(1, 1, &[255, 255, 255, 255]);

    let width = 1.0;
    let height = 1.0;
    let length = 1.0;
    let indices = [0u16, 1, 2, 0, 2, 3];

    let vertices = [
        vec3(-width / 2., height / 2., -length / 2.),
        vec3(-width / 2., height / 2., length / 2.),
        vec3(width / 2., height / 2., length / 2.),
        vec3(width / 2., height / 2., -length / 2.),
    ];
    let uvs = [vec2(0., 1.), vec2(0., 0.), vec2(1., 0.), vec2(1., 1.)];
    let normals = [
        vec3(0., 1., 0.),
        vec3(0., 1., 0.),
        vec3(0., 1., 0.),
        vec3(0., 1., 0.),
    ];

    let vertex_buffer = ctx.new_buffer(
        BufferType::VertexBuffer,
        BufferUsage::Immutable,
        BufferSource::slice(&vertices),
    );
    let normals_buffer = ctx.new_buffer(
        BufferType::VertexBuffer,
        BufferUsage::Immutable,
        BufferSource::slice(&normals),
    );
    let uvs_buffer = ctx.new_buffer(
        BufferType::VertexBuffer,
        BufferUsage::Immutable,
        BufferSource::slice(&uvs),
    );
    let index_buffer = ctx.new_buffer(
        BufferType::IndexBuffer,
        BufferUsage::Immutable,
        BufferSource::slice(&indices),
    );
    let bindings = Bindings {
        vertex_buffers: vec![vertex_buffer, uvs_buffer, normals_buffer],
        index_buffer,
        images: vec![white_texture, white_texture],
    };

    Model {
        bindings: vec![bindings],
    }
}

use crate::quad_gl::QuadGl;

pub struct SpriteLayer {
    gl: QuadGl,
    render_state: RenderState,
}

impl SpriteLayer {
    pub fn new(gl: QuadGl, render_state: RenderState) -> SpriteLayer {
        SpriteLayer { gl, render_state }
    }

    pub fn gl(&mut self) -> &mut QuadGl {
        &mut self.gl
    }
}

pub struct SceneGraph {
    models: Vec<(Model, Mat4)>,
    layers_cache: Vec<QuadGl>,
    default_material: Material,
}

impl SceneGraph {
    pub fn new(ctx: &mut dyn miniquad::RenderingBackend) -> SceneGraph {
        let shader = ctx
            .new_shader(
                ShaderSource {
                    glsl_vertex: Some(shader::VERTEX),
                    glsl_fragment: Some(shader::FRAGMENT),
                    metal_shader: None,
                },
                shader::meta(),
            )
            .unwrap_or_else(|e| panic!("Failed to load shader: {}", e));

        let default_material = Material::new2(
            ctx,
            shader,
            PipelineParams {
                depth_test: Comparison::LessOrEqual,
                depth_write: true,
                ..Default::default()
            },
            vec![],
            vec![],
        )
        .unwrap();

        SceneGraph {
            models: vec![],
            layers_cache: vec![QuadGl::new(ctx), QuadGl::new(ctx), QuadGl::new(ctx)],
            default_material,
        }
    }

    pub fn add_model(&mut self, model: Model) -> usize {
        self.models.push((model, Mat4::IDENTITY));
        self.models.len() - 1
    }

    pub fn fullscreen_canvas(&mut self) -> SpriteLayer {
        fn pixel_perfect_render_state() -> RenderState {
            let (w, h) = (
                crate::window::screen_width(),
                crate::window::screen_height(),
            );
            RenderState {
                camera: crate::camera::Camera::Camera2D {
                    rotation: 0.,
                    zoom: vec2(1. / w * 2., -1. / h * 2.),
                    target: vec2(w / 2., h / 2.),
                    offset: vec2(0., 0.),
                },
                ..Default::default()
            }
        }

        let render_state = pixel_perfect_render_state();
        let mut gl = self.layers_cache.pop().unwrap();
        gl.render_pass(None);

        SpriteLayer::new(gl, render_state)
    }

    pub fn canvas(&mut self, render_state: RenderState) -> SpriteLayer {
        let mut gl = self.layers_cache.pop().unwrap();
        let render_pass = render_state.render_target.as_ref().map(|rt| rt.render_pass);
        gl.render_pass(render_pass);

        SpriteLayer::new(gl, render_state)
    }

    pub fn clear(&mut self, color: Color) {
        let ctx = &mut get_context().quad_context;
        let clear = PassAction::clear_color(color.r, color.g, color.b, color.a);

        ctx.begin_default_pass(clear);
        ctx.end_render_pass();
    }

    pub fn clear2(&mut self, render_state: &RenderState, color: Color) {
        let ctx = &mut get_context().quad_context;
        let clear = PassAction::clear_color(color.r, color.g, color.b, color.a);

        if let Some(pass) = render_state.render_target.as_ref().map(|rt| rt.render_pass) {
            ctx.begin_pass(Some(pass), clear);
        } else {
            ctx.begin_default_pass(clear);
        }
        ctx.end_render_pass();
    }

    pub fn draw_canvas(&mut self, mut canvas: SpriteLayer) {
        let context = &mut *get_context().quad_context;

        let (_width, _height) = miniquad::window::screen_size();

        let screen_mat = //glam::Mat4::orthographic_rh_gl(0., width, height, 0., -1., 1.);
            canvas.render_state.matrix();
        canvas.gl().draw(context, screen_mat);

        self.layers_cache.push(canvas.gl);
    }

    pub fn draw_model(&mut self, render_state: &mut RenderState, model: &Model, transform: Mat4) {
        // unsafe {
        //     miniquad::gl::glPolygonMode(miniquad::gl::GL_FRONT_AND_BACK, miniquad::gl::GL_LINE);
        // }
        let white_texture = get_context().white_texture;
        let ctx = &mut *get_context().quad_context;
        //let projection = self.camera.matrix();

        // let pass = get_context().gl.get_active_render_pass();
        if let Some(pass) = render_state.render_target.as_ref().map(|rt| rt.render_pass) {
            ctx.begin_pass(Some(pass), PassAction::Nothing);
        } else {
            ctx.begin_default_pass(PassAction::Nothing);
        }

        if let Some(ref material) = render_state.material {
            ctx.apply_pipeline(&material.pipeline_3d);
        } else {
            ctx.apply_pipeline(&self.default_material.pipeline_3d);
        }

        let mut bindings = model.bindings.clone();
        for mut bindings in bindings {
            if let Some(ref mut material) = render_state.material {
                bindings.images[0] = material
                    .textures_data
                    .get("Texture")
                    .copied()
                    .unwrap_or(white_texture)
            }
            ctx.apply_bindings(&bindings);

            let projection = render_state.matrix();
            let time = (crate::time::get_time()) as f32;
            let time = glam::vec4(time, time.sin(), time.cos(), 0.);

            if let Some(ref mut material) = render_state.material {
                material.set_uniform("Projection", projection);
                material.set_uniform("Model", transform);
                material.set_uniform("_Time", time);

                ctx.apply_uniforms_from_bytes(
                    material.uniforms_data.as_ptr(),
                    material.uniforms_data.len(),
                );
            } else {
                ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms {
                    projection,
                    model: transform,
                }));
            }

            let buffer_size = ctx.buffer_size(bindings.index_buffer) as i32 / 2;
            ctx.draw(0, buffer_size, 1);
        }
        ctx.end_render_pass();

        // unsafe {
        //     use miniquad::gl;
        //     gl::glPolygonMode(gl::GL_FRONT_AND_BACK, gl::GL_FILL);
        // }
    }

    pub fn set_transform(&mut self, model: usize, transform: Mat4) {
        self.models[model].1 = transform;
    }
}

mod shader {
    use miniquad::{ShaderMeta, UniformBlockLayout, UniformDesc, UniformType};

    pub const VERTEX: &str = r#"#version 100
    attribute vec3 in_position;
    attribute vec2 in_uv;
    attribute vec3 in_normal;

    varying lowp vec4 out_color;
    varying lowp vec2 out_uv;

    uniform mat4 Model;
    uniform mat4 Projection;

    void main() {
        out_color = vec4(dot(in_normal, vec3(0.0, 1.0, 0.0)), dot(in_normal, vec3(0.0, -1.0, 0.0)), dot(in_normal, vec3(-0.2, -0.8, -0.3)), 1);
        gl_Position = Projection * Model * vec4(in_position, 1);
        out_uv = in_uv;
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec4 out_color;
    varying lowp vec2 out_uv;

    lowp float chessboard(lowp vec2 uv)
    {
	    uv = floor(uv * 2.0);
        return mod(uv.x + uv.y, 2.0);
    }

    void main() {
        gl_FragColor = vec4(1.0, 0.0, 0.0, 1) * (max(out_color.x, 0.0) + max(out_color.y, 0.0));
    }"#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["Texture".to_string()],
            uniforms: UniformBlockLayout {
                uniforms: vec![
                    UniformDesc::new("Projection", UniformType::Mat4),
                    UniformDesc::new("Model", UniformType::Mat4),
                ],
            },
        }
    }

    #[repr(C)]
    pub struct Uniforms {
        pub projection: crate::math::Mat4,
        pub model: crate::math::Mat4,
    }
}
