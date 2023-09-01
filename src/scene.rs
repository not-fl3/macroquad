use crate::{
    camera::{self, Camera},
    color::Color,
    error::Error,
    file::load_file,
    material::Material,
    math::{vec2, vec3, Mat4, Quat, Vec3},
    text,
    window::miniquad::*,
    Context3,
};

use std::sync::{Arc, Mutex};

struct NodeData {
    pipeline: Pipeline,
    color: [f32; 4],
    vertex_buffers: Vec<miniquad::BufferId>,
    index_buffer: miniquad::BufferId,
    base_color_texture: Option<miniquad::TextureId>,
    emissive_texture: Option<miniquad::TextureId>,
    normal_texture: Option<miniquad::TextureId>,
    occlusion_texture: Option<miniquad::TextureId>,
    metallic_roughness_texture: Option<miniquad::TextureId>,
    material: [f32; 4],
}
struct Node {
    name: String,
    data: Vec<NodeData>,
    transform: Transform,
}
pub struct Model {
    nodes: Vec<Node>,
}

impl Scene {
    pub fn square(&mut self) -> Model {
        let mut quad_ctx = self.quad_ctx.lock().unwrap();
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
        // let bindings = (
        //     self.default_material.pipeline_3d,
        //     [1., 1., 1., 1.],
        //     Bindings {
        //         vertex_buffers: vec![vertex_buffer, uvs_buffer, normals_buffer],
        //         index_buffer,
        //         images: vec![self.white_texture, self.white_texture],
        //     },
        // );

        // Model {
        //     nodes: vec![Node {
        //         name: "root".to_string(),
        //         data: vec![bindings],
        //         transform: Transform::default(),
        //     }],
        // }
        unimplemented!()
    }
}

pub struct Scene {
    pub(crate) quad_ctx: Arc<Mutex<Box<dyn miniquad::RenderingBackend>>>,
    pub(crate) fonts_storage: Arc<Mutex<text::FontsStorage>>,

    pub(crate) cameras: Vec<camera::Camera>,
    pub(crate) models: Vec<(Model, Transform)>,

    pub(crate) white_texture: TextureId,
    pub(crate) black_texture: TextureId,
    //pub(crate) default_material: Material,
}

impl crate::Context3 {
    pub async fn load_gltf(&self, path: &str) -> Result<Model, Error> {
        use nanogltf::{utils, Gltf};
        let mut ctx = self.quad_ctx.lock().unwrap();

        let bytes = crate::file::load_string(path).await?;

        let gltf = Gltf::from_json(&bytes).unwrap();
        //println!("{:#?}", &gltf);
        let buffers = gltf
            .buffers
            .iter()
            .map(|buffer| {
                let bytes = match utils::parse_uri(&buffer.uri) {
                    utils::UriData::Bytes(bytes) => bytes,
                    _ => unimplemented!(),
                };
                bytes
            })
            .collect::<Vec<_>>();

        assert!(gltf.scenes.len() == 1);

        let mut textures = vec![];
        for image in &gltf.images {
            let source = utils::image_source(&gltf, image);
            let bytes: &[u8] = match source {
                utils::ImageSource::Bytes(ref bytes) => bytes,
                utils::ImageSource::Slice {
                    buffer,
                    offset,
                    length,
                } => &buffers[0][offset..offset + length],
            };
            let image = nanoimage::decode(&bytes).unwrap();
            let texture =
                ctx.new_texture_from_rgba8(image.width as u16, image.height as u16, &image.data);
            ctx.texture_set_wrap(texture, TextureWrap::Repeat);
            textures.push(texture);
        }

        let mut nodes = vec![];
        let scene = &gltf.scenes[0];
        for node in &scene.nodes {
            let node = &gltf.nodes[*node];
            if node.children.len() != 0 {
                continue;
            }
            let translation = node
                .translation
                .map_or(Vec3::ZERO, |t| vec3(t[0] as f32, t[1] as f32, t[2] as f32));
            let rotation = node.rotation.map_or(Quat::IDENTITY, |t| {
                Quat::from_xyzw(t[0] as f32, t[1] as f32, t[2] as f32, t[3] as f32)
            });
            let scale = node
                .scale
                .map_or(Vec3::ZERO, |t| vec3(t[0] as f32, t[1] as f32, t[2] as f32));
            let transform = Transform {
                translation,
                rotation,
                scale,
            };
            let mesh = node.mesh.unwrap();
            let mesh = &gltf.meshes[mesh];
            let mut bindings = Vec::new();

            for primitive in &mesh.primitives {
                let material = &gltf.materials[primitive.material.unwrap()];
                let color = material.pbr_metallic_roughness.base_color_factor;

                let base_color_texture = &material.pbr_metallic_roughness.base_color_texture;
                let base_color_texture = base_color_texture.as_ref().map(|t| textures[t.index]);
                let metallic_roughness_texture = material
                    .pbr_metallic_roughness
                    .metallic_roughness_texture
                    .as_ref()
                    .map(|t| textures[t.index]);
                let emissive_texture = material
                    .emissive_texture
                    .as_ref()
                    .map(|t| textures[t.index]);
                let occlusion_texture = material
                    .occlusion_texture
                    .as_ref()
                    .map(|t| textures[t.index]);
                let normal_texture = material.normal_texture.as_ref().map(|t| textures[t.index]);
                let color = [
                    color[0] as f32,
                    color[1] as f32,
                    color[2] as f32,
                    color[3] as f32,
                ];
                let indices = utils::attribute_bytes(&gltf, primitive.indices.unwrap());
                let indices = &buffers[indices.0][indices.1..indices.1 + indices.2];
                let vertices = utils::attribute_bytes(&gltf, primitive.attributes["POSITION"]);
                let vertices = &buffers[vertices.0][vertices.1..vertices.1 + vertices.2];
                let uvs = utils::attribute_bytes(&gltf, primitive.attributes["TEXCOORD_0"]);
                let uvs = &buffers[uvs.0][uvs.1..uvs.1 + uvs.2];
                let normals = utils::attribute_bytes(&gltf, primitive.attributes["NORMAL"]);
                let normals = &buffers[normals.0][normals.1..normals.1 + normals.2];

                let vertex_buffer =
                    ctx.new_buffer(BufferType::VertexBuffer, BufferUsage::Immutable, unsafe {
                        BufferSource::pointer(vertices.as_ptr(), vertices.len(), 4 * 3)
                    });
                let normals_buffer =
                    ctx.new_buffer(BufferType::VertexBuffer, BufferUsage::Immutable, unsafe {
                        BufferSource::pointer(normals.as_ptr(), normals.len(), 4 * 3)
                    });
                let uvs_buffer =
                    ctx.new_buffer(BufferType::VertexBuffer, BufferUsage::Immutable, unsafe {
                        BufferSource::pointer(uvs.as_ptr(), uvs.len(), 4 * 2)
                    });
                let index_buffer =
                    ctx.new_buffer(BufferType::IndexBuffer, BufferUsage::Immutable, unsafe {
                        BufferSource::pointer(indices.as_ptr(), indices.len(), 2)
                    });

                let mut defines = vec![];
                if normal_texture.is_some() {
                    defines.push("HAS_NORMAL_MAP".to_string());
                }
                if metallic_roughness_texture.is_some() {
                    defines.push("HAS_METALLIC_ROUGHNESS_MAP".to_string());
                }

                let shader = shadermagic::transform(
                    shader::FRAGMENT,
                    shader::VERTEX,
                    &shader::meta(),
                    &shadermagic::Options {
                        defines,
                        ..Default::default()
                    },
                )
                .unwrap();
                let shader = shadermagic::choose_appropriate_shader(&shader, &ctx.info());
                if let miniquad::ShaderSource::Glsl { fragment, vertex } = shader {
                    //println!("{}", vertex);
                };
                let shader = ctx
                    .new_shader(shader, shader::meta())
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

                bindings.push(NodeData {
                    pipeline,
                    color,
                    vertex_buffers: vec![vertex_buffer, uvs_buffer, normals_buffer],
                    index_buffer,
                    base_color_texture,
                    emissive_texture,
                    normal_texture,
                    occlusion_texture,
                    metallic_roughness_texture,
                    material: [
                        material.pbr_metallic_roughness.metallic_factor as f32,
                        material.pbr_metallic_roughness.roughness_factor as f32,
                        0.,
                        0.,
                    ],
                });
            }

            nodes.push(Node {
                name: node
                    .name
                    .clone()
                    .unwrap_or("unnamed".to_string())
                    .to_owned(),
                data: bindings,
                transform,
            });
        }

        Ok(Model { nodes })
    }

    pub fn load_cubemap(&self, cubemap: &[&[u8]]) -> crate::cubemap::Cubemap {
        crate::cubemap::Cubemap::new(&mut **self.quad_ctx.lock().unwrap(), cubemap)
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

pub struct ModelHandle(usize);
impl Scene {
    pub fn borrow_model(&mut self, h: &ModelHandle) -> &mut Transform {
        &mut self.models[h.0].1
    }
    pub fn translation(&mut self, h: &ModelHandle) -> Vec3 {
        self.models[h.0].1.translation
    }
    pub fn rotation(&mut self, h: &ModelHandle) -> Quat {
        self.models[h.0].1.rotation
    }
    pub fn update(&mut self, h: &ModelHandle, f: impl Fn(&mut Transform)) {
        f(&mut self.models[h.0].1)
    }

    pub fn update_child(&mut self, h: &ModelHandle, name: &str, f: impl Fn(&mut Transform)) {
        let model = &mut self.models[h.0];
        for child in &mut model.0.nodes {
            if child.name == name {
                f(&mut child.transform)
            }
        }
    }
}

#[derive(Clone)]
pub struct CameraHandle(usize);

impl Scene {
    pub fn update_camera(&mut self, h: &CameraHandle, f: impl Fn(&mut Camera)) {
        f(&mut self.cameras[h.0]);
    }
}
impl Scene {
    pub(crate) fn new(
        ctx: Arc<Mutex<Box<dyn miniquad::RenderingBackend>>>,
        fonts_storage: Arc<Mutex<text::FontsStorage>>,
    ) -> Scene {
        let quad_ctx = ctx.clone();
        let mut ctx = ctx.lock().unwrap();
        // let shader = ctx
        //     .new_shader(
        //         ShaderSource::Glsl {
        //             vertex: shader::VERTEX,
        //             fragment: shader::FRAGMENT,
        //         },
        //         shader::meta(),
        //     )
        //     .unwrap_or_else(|e| panic!("Failed to load shader: {}", e));

        // let default_material = Material::new2(
        //     &mut **ctx,
        //     shader,
        //     PipelineParams {
        //         depth_test: Comparison::LessOrEqual,
        //         depth_write: true,
        //         ..Default::default()
        //     },
        //     vec![],
        //     vec![],
        // )
        // .unwrap();

        Scene {
            white_texture: ctx.new_texture_from_rgba8(1, 1, &[255, 255, 255, 255]),
            black_texture: ctx.new_texture_from_rgba8(1, 1, &[0, 0, 0, 0]),
            fonts_storage: fonts_storage.clone(),

            cameras: vec![],
            models: vec![],
            //default_material,
            quad_ctx,
        }
    }
}

impl Scene {
    pub fn add_camera(&mut self, camera: camera::Camera) -> CameraHandle {
        self.cameras.push(camera);
        CameraHandle(self.cameras.len() - 1)
    }

    pub fn add_model(&mut self, model: Model) -> ModelHandle {
        self.models.push((
            model,
            Transform {
                translation: vec3(0.0, 0.0, 0.0),
                scale: vec3(1., 1., 1.),
                rotation: Quat::IDENTITY,
            },
        ));
        ModelHandle(self.models.len() - 1)
    }

    // pub fn fullscreen_canvas(&self, ix: usize) -> sprite_layer::SpriteLayer {
    //     // fn pixel_perfect_render_state() -> RenderState {
    //     //     let (w, h) = (
    //     //         crate::window::screen_width(),
    //     //         crate::window::screen_height(),
    //     //     );
    //     //     RenderState {
    //     //         camera: crate::camera::Camera::Camera2D {
    //     //             rotation: 0.,
    //     //             zoom: vec2(1. / w * 2., -1. / h * 2.),
    //     //             target: vec2(w / 2., h / 2.),
    //     //             offset: vec2(0., 0.),
    //     //         },
    //     //         ..Default::default()
    //     //     }
    //     // }

    //     //let render_state = pixel_perfect_render_state();
    //     // self.data.layers.lock()[ix].render_pass(None);
    //     // self.data.layers.lock()[ix].clear_draw_calls();

    //     //SpriteLayer::new(self.ctx.clone(), ix)
    //     unimplemented!()
    // }

    // pub fn canvas(&self, render_state: RenderState) -> SpriteLayer {
    //     let mut gl = self.layers.lock()..pop().unwrap();
    //     let render_pass = render_state.render_target.as_ref().map(|rt| rt.render_pass);
    //     gl.render_pass(render_pass);

    //     SpriteLayer::new(self, gl, render_state)
    // }

    pub fn clear(&self, color: Color) {
        let mut ctx = self.quad_ctx.lock().unwrap();
        let clear = PassAction::clear_color(color.r, color.g, color.b, color.a);

        ctx.begin_default_pass(clear);
        ctx.end_render_pass();
    }

    // pub fn clear2(&mut self, ctx: &Context2, color: Color) {
    //     let mut ctx = self.quad_ctx.lock().unwrap();
    //     let clear = PassAction::clear_color(color.r, color.g, color.b, color.a);

    //     if let Some(pass) = render_state.render_target.as_ref().map(|rt| rt.render_pass) {
    //         ctx.begin_pass(Some(pass), clear);
    //     } else {
    //         ctx.begin_default_pass(clear);
    //     }
    //     ctx.end_render_pass();
    // }

    pub(crate) fn draw_canvas(&self, ix: usize) {
        // let mut ctx = self.data.quad_ctx.lock().unwrap();

        // let (width, height) = miniquad::window::screen_size();

        // let screen_mat = glam::Mat4::orthographic_rh_gl(0., width, height, 0., -1., 1.);
        // let canvas = &mut self.data.layers.lock()[ix];
        // canvas.draw(&mut **ctx, screen_mat);

        unimplemented!()
    }

    pub(crate) fn draw_model(
        ctx: &mut miniquad::Context,
        white_texture: TextureId,
        black_texture: TextureId,
        model: &Model,
        camera: &camera::Camera,
        transform: Mat4,
    ) {
        // unsafe {
        //     miniquad::gl::glPolygonMode(miniquad::gl::GL_FRONT_AND_BACK, miniquad::gl::GL_LINE);
        // }

        if let Some(pass) = camera.render_target.as_ref().map(|rt| rt.render_pass) {
            ctx.begin_pass(Some(pass), PassAction::Nothing);
        } else {
            ctx.begin_default_pass(PassAction::Nothing);
        }

        for node in &model.nodes {
            for bindings in &node.data {
                let cubemap = match camera.environment {
                    crate::camera::Environment::Skybox(ref cubemap) => cubemap.texture,
                    _ => unimplemented!(),
                };
                let images = [
                    bindings.base_color_texture.unwrap_or(white_texture),
                    bindings.emissive_texture.unwrap_or(black_texture),
                    bindings.occlusion_texture.unwrap_or(white_texture),
                    bindings.normal_texture.unwrap_or(white_texture),
                    bindings.metallic_roughness_texture.unwrap_or(white_texture),
                    cubemap,
                ];
                ctx.apply_pipeline(&bindings.pipeline);
                ctx.apply_bindings_from_slice(
                    &bindings.vertex_buffers,
                    bindings.index_buffer,
                    &images,
                );

                let (proj, view) = camera.proj_view();
                let projection = proj * view;
                let time = (crate::time::get_time()) as f32;
                let time = glam::vec4(time, time.sin(), time.cos(), 0.);

                let model = transform
                    * (Mat4::from_translation(node.transform.translation)
                        * Mat4::from_quat(node.transform.rotation));
                let model_inverse = model.inverse();
                ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms {
                    projection,
                    model,
                    model_inverse,
                    color: bindings.color,
                    material: bindings.material,
                    camera_pos: camera.position(),
                }));

                let buffer_size = ctx.buffer_size(bindings.index_buffer) as i32 / 2;
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

    pub fn draw(&mut self) {
        for camera in &mut self.cameras {
            let (proj, view) = camera.proj_view();
            if let crate::camera::Environment::Skybox(ref mut cubemap) = camera.environment {
                cubemap.draw(&mut **self.quad_ctx.lock().unwrap(), &proj, &view);
            }
            for (model, t) in &self.models {
                let mat = t.matrix();
                Scene::draw_model(
                    &mut **self.quad_ctx.lock().unwrap(),
                    self.white_texture,
                    self.black_texture,
                    model,
                    camera,
                    mat,
                );
            }
        }
    }
}

mod shader {
    use crate::math::Vec3;
    use miniquad::{ShaderMeta, UniformBlockLayout, UniformDesc, UniformType};

    pub const VERTEX: &str = include_str!("vertex.glsl");
    pub const FRAGMENT: &str = include_str!("fragment.glsl");
    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec![
                "Albedo".to_string(),
                "Emissive".to_string(),
                "Occlusion".to_string(),
                "Normal".to_string(),
                "MetallicRoughness".to_string(),
                "Environment".to_string(),
            ],
            uniforms: UniformBlockLayout {
                uniforms: vec![
                    UniformDesc::new("Projection", UniformType::Mat4),
                    UniformDesc::new("Model", UniformType::Mat4),
                    UniformDesc::new("ModelInverse", UniformType::Mat4),
                    UniformDesc::new("Color", UniformType::Float4),
                    UniformDesc::new("Material", UniformType::Float4),
                    UniformDesc::new("CameraPosition", UniformType::Float3),
                ],
            },
        }
    }

    #[repr(C)]
    pub struct Uniforms {
        pub projection: glam::Mat4,
        pub model: glam::Mat4,
        pub model_inverse: glam::Mat4,
        pub color: [f32; 4],
        pub material: [f32; 4], // metallic, roughness, 0, 0,
        pub camera_pos: glam::Vec3,
    }
}
