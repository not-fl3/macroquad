use crate::{
    camera::{self, Camera},
    color::Color,
    error::Error,
    file::load_file,
    material::Material,
    math::{vec2, vec3, Mat4, Quat, Vec3},
    text,
    window::miniquad::*,
    Context2, Context3,
};

use parking_lot::{Mutex, MutexGuard};
use std::sync::Arc;

struct Node {
    name: String,
    data: Vec<(Pipeline, [f32; 4], Bindings)>,
    transform: Transform,
}
pub struct Model {
    nodes: Vec<Node>,
}

pub fn square(ctx: &Context2) -> Model {
    let mut quad_ctx = ctx.scene.quad_context.lock();
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

    let vertex_buffer = quad_ctx.new_buffer(
        BufferType::VertexBuffer,
        BufferUsage::Immutable,
        BufferSource::slice(&vertices),
    );
    let normals_buffer = quad_ctx.new_buffer(
        BufferType::VertexBuffer,
        BufferUsage::Immutable,
        BufferSource::slice(&normals),
    );
    let uvs_buffer = quad_ctx.new_buffer(
        BufferType::VertexBuffer,
        BufferUsage::Immutable,
        BufferSource::slice(&uvs),
    );
    let index_buffer = quad_ctx.new_buffer(
        BufferType::IndexBuffer,
        BufferUsage::Immutable,
        BufferSource::slice(&indices),
    );
    let bindings = (
        ctx.scene.default_material.pipeline_3d,
        [1., 1., 1., 1.],
        Bindings {
            vertex_buffers: vec![vertex_buffer, uvs_buffer, normals_buffer],
            index_buffer,
            images: vec![ctx.scene.white_texture, ctx.scene.white_texture],
        },
    );

    Model {
        nodes: vec![Node {
            name: "root".to_string(),
            data: vec![bindings],
            transform: Transform::default(),
        }],
    }
}

use crate::quad_gl::QuadGl;

pub struct SpriteLayer {
    pub(crate) ctx: Arc<Context2>,
    ix: usize,
}

impl SpriteLayer {
    pub(crate) fn new(ctx: Arc<Context2>, ix: usize) -> SpriteLayer {
        SpriteLayer { ctx, ix }
    }

    pub(crate) fn gl<'a>(&'a mut self) -> impl std::ops::DerefMut<Target = QuadGl> + 'a {
        MutexGuard::map(self.ctx.scene.layers.lock(), |x| &mut x[self.ix])
    }
}

pub(crate) struct SceneData {
    pub(crate) quad_context: Mutex<Box<dyn miniquad::RenderingBackend>>,
    pub(crate) fonts_storage: Mutex<text::FontsStorage>,

    pub(crate) cameras: Mutex<Vec<camera::Camera>>,
    pub(crate) models: Mutex<Vec<(Model, Transform)>>,
    pub(crate) layers: Mutex<Vec<QuadGl>>,

    pub(crate) white_texture: TextureId,
    pub(crate) default_material: Material,
}

pub struct Scene<'a> {
    pub(crate) data: &'a SceneData,
    pub(crate) ctx: Arc<Context2>,
}

impl Context3 {
    pub async fn load_gltf(&self, path: &str) -> Result<Model, Error> {
        let mut ctx = self.ctx.scene.quad_context.lock();

        let bytes = load_file(path).await?;

        let (gltf, buffers, _images) = gltf::import_slice(&bytes).unwrap();

        assert!(gltf.scenes().len() == 1);
        let mut nodes = vec![];
        let mut texture = None;
        for image in gltf.images() {
            println!("image: {:?}", image.name());
            let view = match image.source() {
                gltf::image::Source::View { view, .. } => view,
                _ => unimplemented!(),
            };
            let buffer = &buffers[view.buffer().index()].0;
            let data = &buffer[view.offset()..view.offset() + view.length()];
            let png = nanoimage::png::decode(data).unwrap();
            texture =
                Some(ctx.new_texture_from_rgba8(png.width as u16, png.height as u16, &png.data));
        }

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

        let pipeline = ctx.new_pipeline_with_params(
            &[
                BufferLayout::default(),
                BufferLayout::default(),
                BufferLayout::default(),
            ],
            &[
                VertexAttribute::with_buffer("in_position", VertexFormat::Float3, 0),
                VertexAttribute::with_buffer("in_uv", VertexFormat::Float2, 1),
                VertexAttribute::with_buffer("in_normal", VertexFormat::Float3, 2),
            ],
            shader,
            PipelineParams {
                depth_test: Comparison::LessOrEqual,
                depth_write: true,
                ..Default::default()
            },
        );

        let scene = gltf.scenes().next().unwrap();
        for node in scene.nodes() {
            let transform = node.transform().decomposed();
            let transform = Transform {
                translation: transform.0.into(),
                rotation: Quat::from_array(transform.1),
                scale: transform.2.into(),
            };
            // println!(
            //     "Node {:?} #{:?} has {} children",
            //     node.name(),
            //     node.index(),
            //     node.children().count(),
            // );

            let mesh = node.mesh().unwrap();

            let mut bindings = Vec::new();

            for primitive in mesh.primitives() {
                let color = primitive
                    .material()
                    .pbr_metallic_roughness()
                    .base_color_factor();
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
                bindings.push((
                    pipeline,
                    color,
                    Bindings {
                        vertex_buffers: vec![vertex_buffer, uvs_buffer, normals_buffer],
                        index_buffer,
                        images: vec![texture.unwrap_or(white_texture), white_texture],
                    },
                ));
            }
            nodes.push(Node {
                name: node.name().unwrap().to_owned(),
                data: bindings,
                transform,
            });
        }

        Ok(Model { nodes })
    }

    pub fn load_cubemap(&self, cubemap: &[&[u8]]) -> crate::cubemap::Cubemap {
        crate::cubemap::Cubemap::new(&mut **self.ctx.scene.quad_context.lock(), cubemap)
    }
}

pub struct Transform {
    pub translation: Vec3,
    pub scale: Vec3,
    pub rotation: Quat,
}
impl Default for Transform {
    fn default() -> Transform {
        Transform {
            translation: vec3(0.0, 0.0, 0.0),
            scale: vec3(1.0, 1.0, 1.0),
            rotation: Quat::IDENTITY,
        }
    }
}
impl Transform {
    pub(crate) fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

pub struct ModelHandle(Arc<Context2>, usize);
impl ModelHandle {
    pub fn translation(&self) -> Vec3 {
        let ix = self.1;
        let mut models = self.0.scene.models.lock();
        models[ix].1.translation
    }
    pub fn rotation(&self) -> Quat {
        let ix = self.1;
        let mut models = self.0.scene.models.lock();
        models[ix].1.rotation
    }
    pub fn update(&self, f: impl Fn(&mut Transform)) {
        let ix = self.1;
        let mut models = self.0.scene.models.lock();
        f(&mut models[ix].1)
    }

    pub fn update_child(&self, name: &str, f: impl Fn(&mut Transform)) {
        let ix = self.1;
        let mut models = self.0.scene.models.lock();
        let model = &mut models[ix];
        for child in &mut model.0.nodes {
            if child.name == name {
                f(&mut child.transform)
            }
        }
    }
}

pub struct CameraHandle(Arc<Context2>, usize);
impl CameraHandle {
    pub fn update(&self, f: impl Fn(&mut Camera)) {
        let ix = self.1;
        let mut cameras = self.0.scene.cameras.lock();
        f(&mut cameras[ix]);
    }
}
impl SceneData {
    pub(crate) fn new(mut ctx: Box<dyn miniquad::RenderingBackend>) -> SceneData {
        let fonts_storage = text::FontsStorage::new(&mut *ctx);

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
            &mut *ctx,
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

        SceneData {
            white_texture: ctx.new_texture_from_rgba8(1, 1, &[255, 255, 255, 255]),
            fonts_storage: Mutex::new(fonts_storage),

            cameras: Mutex::new(vec![]),
            models: Mutex::new(vec![]),
            layers: Mutex::new(vec![
                QuadGl::new(&mut *ctx),
                QuadGl::new(&mut *ctx),
                QuadGl::new(&mut *ctx),
            ]),
            default_material,

            quad_context: Mutex::new(ctx),
        }
    }
}

impl<'a> Scene<'a> {
    pub fn add_camera(&self, camera: camera::Camera) -> CameraHandle {
        let mut cameras = self.data.cameras.lock();
        cameras.push(camera);
        CameraHandle(self.ctx.clone(), cameras.len() - 1)
    }

    pub fn add_model(&self, model: Model) -> ModelHandle {
        self.data.models.lock().push((
            model,
            Transform {
                translation: vec3(0.0, 0.0, 0.0),
                scale: vec3(1., 1., 1.),
                rotation: Quat::IDENTITY,
            },
        ));
        ModelHandle(self.ctx.clone(), self.data.models.lock().len() - 1)
    }

    pub fn fullscreen_canvas(&self, ix: usize) -> SpriteLayer {
        // fn pixel_perfect_render_state() -> RenderState {
        //     let (w, h) = (
        //         crate::window::screen_width(),
        //         crate::window::screen_height(),
        //     );
        //     RenderState {
        //         camera: crate::camera::Camera::Camera2D {
        //             rotation: 0.,
        //             zoom: vec2(1. / w * 2., -1. / h * 2.),
        //             target: vec2(w / 2., h / 2.),
        //             offset: vec2(0., 0.),
        //         },
        //         ..Default::default()
        //     }
        // }

        //let render_state = pixel_perfect_render_state();
        self.data.layers.lock()[ix].render_pass(None);
        self.data.layers.lock()[ix].clear_draw_calls();

        SpriteLayer::new(self.ctx.clone(), ix)
    }

    // pub fn canvas(&self, render_state: RenderState) -> SpriteLayer {
    //     let mut gl = self.layers.lock()..pop().unwrap();
    //     let render_pass = render_state.render_target.as_ref().map(|rt| rt.render_pass);
    //     gl.render_pass(render_pass);

    //     SpriteLayer::new(self, gl, render_state)
    // }

    pub fn clear(&self, color: Color) {
        let mut ctx = self.data.quad_context.lock();
        let clear = PassAction::clear_color(color.r, color.g, color.b, color.a);

        ctx.begin_default_pass(clear);
        ctx.end_render_pass();
    }

    // pub fn clear2(&mut self, ctx: &Context2, color: Color) {
    //     let mut ctx = self.quad_context.lock();
    //     let clear = PassAction::clear_color(color.r, color.g, color.b, color.a);

    //     if let Some(pass) = render_state.render_target.as_ref().map(|rt| rt.render_pass) {
    //         ctx.begin_pass(Some(pass), clear);
    //     } else {
    //         ctx.begin_default_pass(clear);
    //     }
    //     ctx.end_render_pass();
    // }

    pub(crate) fn draw_canvas(&self, ix: usize) {
        let mut ctx = self.data.quad_context.lock();

        let (width, height) = miniquad::window::screen_size();

        let screen_mat = glam::Mat4::orthographic_rh_gl(0., width, height, 0., -1., 1.);
        let canvas = &mut self.data.layers.lock()[ix];
        canvas.draw(&mut **ctx, screen_mat);
    }

    pub(crate) fn draw_model(&self, model: &Model, camera: &camera::Camera, transform: Mat4) {
        // unsafe {
        //     miniquad::gl::glPolygonMode(miniquad::gl::GL_FRONT_AND_BACK, miniquad::gl::GL_LINE);
        // }
        //let white_texture = get_context().white_texture;
        let white_texture = self.data.white_texture;
        let mut ctx = self.data.quad_context.lock();
        //let projection = self.camera.matrix();

        // let pass = get_context().gl.get_active_render_pass();
        if let Some(pass) = camera.render_target.as_ref().map(|rt| rt.render_pass) {
            ctx.begin_pass(Some(pass), PassAction::Nothing);
        } else {
            ctx.begin_default_pass(PassAction::Nothing);
        }

        for node in &model.nodes {
            for bindings in &node.data {
                ctx.apply_pipeline(&bindings.0);
                ctx.apply_bindings(&bindings.2);

                let (proj, view) = camera.proj_view();
                let projection = proj * view;
                let time = (crate::time::get_time()) as f32;
                let time = glam::vec4(time, time.sin(), time.cos(), 0.);

                ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms {
                    projection,
                    model: transform
                        * (Mat4::from_translation(node.transform.translation)
                            * Mat4::from_quat(node.transform.rotation)),
                    color: bindings.1,
                }));
                //}

                let buffer_size = ctx.buffer_size(bindings.2.index_buffer) as i32 / 2;
                ctx.draw(0, buffer_size, 1);
            }
        }
        ctx.end_render_pass();

        // unsafe {
        //     use miniquad::gl;
        //     gl::glPolygonMode(gl::GL_FRONT_AND_BACK, gl::GL_FILL);
        // }
    }

    // pub fn set_transform(&self, model: usize, transform: Mat4) {
    //     self.models[model].1 = transform;
    // }
}

mod shader {
    use miniquad::{ShaderMeta, UniformBlockLayout, UniformDesc, UniformType};

    pub const VERTEX: &str = r#"#version 100
    attribute vec3 in_position;
    attribute vec2 in_uv;
    attribute vec4 in_normal;

    varying lowp vec2 out_uv;
    varying lowp vec4 out_color;

    uniform mat4 Model;
    uniform mat4 Projection;
    uniform vec4 Color;

    void main() {
        gl_Position = Projection * Model * vec4(in_position, 1);
        out_uv = in_uv;
        out_color = Color;
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec2 out_uv;
    varying lowp vec4 out_color;

    uniform sampler2D Texture;

    void main() {
        gl_FragColor = texture2D(Texture, out_uv) * out_color;
    }"#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["Texture".to_string()],
            uniforms: UniformBlockLayout {
                uniforms: vec![
                    UniformDesc::new("Projection", UniformType::Mat4),
                    UniformDesc::new("Model", UniformType::Mat4),
                    UniformDesc::new("Color", UniformType::Float4),
                ],
            },
        }
    }

    #[repr(C)]
    pub struct Uniforms {
        pub projection: crate::math::Mat4,
        pub model: crate::math::Mat4,
        pub color: [f32; 4],
    }
}
