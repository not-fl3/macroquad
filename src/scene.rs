use crate::{
    camera::{self, Camera},
    color::Color,
    error::Error,
    file::load_file,
    image,
    material::Material,
    math::{vec2, vec3, Mat4, Quat, Vec2, Vec3},
    telemetry, text,
    window::miniquad::*,
    Context3,
};

use std::sync::{Arc, Mutex};

pub mod frustum;

#[derive(Clone)]
pub struct NodeData {
    pub vertex_buffers: Vec<miniquad::BufferId>,
    pub index_buffer: miniquad::BufferId,

    pub(crate) pipeline: Pipeline,
    pub(crate) color: [f32; 4],
    pub(crate) base_color_texture: Option<miniquad::TextureId>,
    pub(crate) emissive_texture: Option<miniquad::TextureId>,
    pub(crate) normal_texture: Option<miniquad::TextureId>,
    pub(crate) occlusion_texture: Option<miniquad::TextureId>,
    pub(crate) metallic_roughness_texture: Option<miniquad::TextureId>,
    pub(crate) material: [f32; 4],
}

#[derive(Clone)]
pub struct Node {
    pub name: String,
    pub data: Vec<NodeData>,
    pub transform: Transform,
}

#[derive(Clone, Copy, Debug)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

#[derive(Clone)]
pub struct Model {
    pub nodes: Vec<Node>,
    pub aabb: AABB,
}

pub struct Model2 {
    pub model: Model,
    pub transform: Transform,
    pub world_aabb: AABB,
}

#[derive(Debug, Clone)]
pub enum ShadowSplit {
    Orthogonal,
    PSSM2,
    PSSM4,
}

#[derive(Clone)]
pub struct ShadowCaster {
    pub direction: Vec3,
    pub split: ShadowSplit,
}

pub struct Scene {
    pub(crate) quad_ctx: Arc<Mutex<Box<dyn miniquad::RenderingBackend>>>,
    pub(crate) fonts_storage: Arc<Mutex<text::FontsStorage>>,

    pub(crate) cameras: Vec<camera::Camera>,
    pub(crate) models: Vec<Model2>,
    pub(crate) shadow_casters: Vec<ShadowCaster>,

    pub(crate) white_texture: TextureId,
    pub(crate) black_texture: TextureId,

    pub(crate) shadowmap: crate::shadowmap::ShadowMap,
    //pub(crate) default_material: Material,
}

impl crate::Context3 {
    pub async fn load_gltf(&self, path: &str) -> Result<Model, Error> {
        use nanogltf::{utils, Gltf};
        use std::borrow::Cow;

        let mut ctx = self.quad_ctx.lock().unwrap();

        let bytes = crate::file::load_string(path).await?;

        let gltf = Gltf::from_json(&bytes).unwrap();
        //println!("{:#?}", &gltf);
        let mut buffers = vec![];
        for buffer in &gltf.buffers {
            let bytes = match utils::parse_uri(&buffer.uri) {
                utils::UriData::Bytes(bytes) => bytes,
                utils::UriData::RelativePath(uri) => {
                    // examples/assets/a.gltf -> examples/assets
                    use std::path::Path;
                    let path = Path::new(&path);
                    let parent = path.parent().map_or("", |parent| parent.to_str().unwrap());
                    crate::file::load_file(&format!("{}/{}", parent, uri)).await?
                }
            };
            buffers.push(bytes);
        }

        assert!(gltf.scenes.len() == 1);

        let mut textures = vec![];
        for image in &gltf.images {
            let source = utils::image_source(&gltf, image);
            let bytes = match source {
                utils::ImageSource::Bytes(ref bytes) => Cow::from(bytes),
                utils::ImageSource::RelativePath(ref uri) => {
                    use std::path::Path;
                    let path = Path::new(&path);
                    let parent = path.parent().map_or("", |parent| parent.to_str().unwrap());
                    Cow::from(crate::file::load_file(&format!("{}/{}", parent, uri)).await?)
                }
                utils::ImageSource::Slice {
                    buffer,
                    offset,
                    length,
                } => Cow::from(&buffers[0][offset..offset + length]),
            };
            let image = image::decode(&bytes).unwrap();
            let texture =
                ctx.new_texture_from_rgba8(image.width as u16, image.height as u16, &image.data);
            ctx.texture_set_wrap(texture, TextureWrap::Repeat, TextureWrap::Repeat);
            textures.push(texture);
        }

        let mut nodes = vec![];
        let scene = &gltf.scenes[0];
        let mut aabb = AABB {
            min: vec3(std::f32::MAX, std::f32::MAX, std::f32::MAX),
            max: vec3(-std::f32::MAX, -std::f32::MAX, -std::f32::MAX),
        };
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
            let scale = node.scale.map_or(vec3(1.0, 1.0, 1.0), |t| {
                vec3(t[0] as f32, t[1] as f32, t[2] as f32)
            });
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

                {
                    let accessor = &gltf.accessors[primitive.attributes["POSITION"]];
                    let matrix = transform.matrix();

                    let min = accessor.min.as_ref().unwrap();
                    let min = vec3(min[0] as f32, min[1] as f32, min[2] as f32);
                    let min = matrix.transform_point3(min);
                    aabb.min = aabb.min.min(min);
                    let max = accessor.max.as_ref().unwrap();
                    let max = vec3(max[0] as f32, max[1] as f32, max[2] as f32);
                    let max = matrix.transform_point3(max);
                    aabb.max = aabb.max.max(max);
                }

                let vertices = utils::attribute_bytes(&gltf, primitive.attributes["POSITION"]);
                let vertices: &[u8] = &buffers[vertices.0][vertices.1..vertices.1 + vertices.2];
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
                        color_blend: Some(BlendState::new(
                            Equation::Add,
                            BlendFactor::Value(BlendValue::SourceAlpha),
                            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                        )),

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
        Ok(Model { nodes, aabb })
    }

    pub async fn load_cubemap(
        &self,
        texture_px: &str,
        texture_nx: &str,
        texture_py: &str,
        texture_ny: &str,
        texture_pz: &str,
        texture_nz: &str,
    ) -> Result<crate::cubemap::Cubemap, crate::Error> {
        let cubemap = [
            &crate::file::load_file(texture_px).await?[..],
            &crate::file::load_file(texture_nx).await?[..],
            &crate::file::load_file(texture_py).await?[..],
            &crate::file::load_file(texture_ny).await?[..],
            &crate::file::load_file(texture_pz).await?[..],
            &crate::file::load_file(texture_nz).await?[..],
        ];
        let mut quad_ctx = self.quad_ctx.lock().unwrap();
        let cubemap = crate::cubemap::Cubemap::new(quad_ctx.as_mut(), &cubemap[..]);
        quad_ctx.texture_set_min_filter(
            cubemap.texture,
            FilterMode::Linear,
            MipmapFilterMode::Linear,
        );
        quad_ctx.texture_generate_mipmaps(cubemap.texture);
        Ok(cubemap)
    }
}

#[derive(Clone, Debug)]
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

impl Model2 {
    fn update_aabb(&mut self) {
        let aabb = self.model.aabb;
        let min = self.transform.matrix().transform_point3(aabb.min);
        let max = self.transform.matrix().transform_point3(aabb.max);
        self.world_aabb = AABB { min, max };
    }
}
#[derive(Clone)]
pub struct ModelHandle(usize);

impl Scene {
    pub fn aabb(&self, h: &ModelHandle) -> AABB {
        self.models[h.0].world_aabb
    }
    pub fn set_translation(&mut self, h: &ModelHandle, pos: Vec3) {
        self.models[h.0].transform.translation = pos;
        self.models[h.0].update_aabb();
    }
    pub fn set_rotation(&mut self, h: &ModelHandle, rotation: Quat) {
        self.models[h.0].transform.rotation = rotation;
        self.models[h.0].update_aabb();
    }
    pub fn set_scale(&mut self, h: &ModelHandle, scale: Vec3) {
        self.models[h.0].transform.scale = scale;
        self.models[h.0].update_aabb();
    }

    pub fn translation(&mut self, h: &ModelHandle) -> Vec3 {
        self.models[h.0].transform.translation
    }
    pub fn rotation(&mut self, h: &ModelHandle) -> Quat {
        self.models[h.0].transform.rotation
    }

    pub fn update_child(&mut self, h: &ModelHandle, name: &str, f: impl Fn(&mut Transform)) {
        let model = &mut self.models[h.0];
        for child in &mut model.model.nodes {
            if child.name == name {
                f(&mut child.transform)
            }
        }
    }
}

// #[derive(Clone)]
// pub struct CameraHandle(usize);

// impl Scene {
//     pub fn camera(&self, h: &CameraHandle) -> &Camera {
//         &self.cameras[h.0]
//     }
//     pub fn camera_mut(&mut self, h: &CameraHandle) -> &mut Camera {
//         &mut self.cameras[h.0]
//     }
// }
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
            shadow_casters: vec![],

            shadowmap: crate::shadowmap::ShadowMap::new(ctx.as_mut()),
            //default_material,
            quad_ctx,
        }
    }
}

impl Scene {
    // pub fn add_camera(&mut self, camera: camera::Camera) -> CameraHandle {
    //     self.cameras.push(camera);
    //     CameraHandle(self.cameras.len() - 1)
    // }

    pub fn add_shadow_caster(&mut self, shadow_caster: ShadowCaster) {
        self.shadow_casters.push(shadow_caster);
    }
    pub fn add_model(&mut self, model: &Model) -> ModelHandle {
        self.models.push(Model2 {
            model: model.clone(),
            transform: Transform {
                translation: vec3(0.0, 0.0, 0.0),
                scale: vec3(1., 1., 1.),
                rotation: Quat::IDENTITY,
            },
            world_aabb: model.aabb,
        });
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
        model: &Model2,
        camera: &camera::Camera,
        shadow_proj: [Mat4; 4],
        shadow_cascades: [f32; 4],
        shadowmap: [TextureId; 4],
        shadow_casters: [i32; 4],
        clipping_planes: [frustum::Plane; 6],
    ) {
        // unsafe {
        //     miniquad::gl::glPolygonMode(miniquad::gl::GL_FRONT_AND_BACK, miniquad::gl::GL_LINE);
        // }

        let transform = model.transform.matrix();
        let aabb = model.world_aabb;
        let model = &model.model;
        if clipping_planes.iter().any(|p| !p.clip(aabb)) {
            return;
        }
        for node in &model.nodes {
            for bindings in &node.data {
                let cubemap = match camera.environment {
                    crate::camera::Environment::Skybox(ref cubemap) => Some(cubemap.texture),
                    _ => None,
                };
                let images = [
                    bindings.base_color_texture.unwrap_or(white_texture),
                    bindings.emissive_texture.unwrap_or(black_texture),
                    bindings.occlusion_texture.unwrap_or(white_texture),
                    bindings.normal_texture.unwrap_or(white_texture),
                    bindings.metallic_roughness_texture.unwrap_or(white_texture),
                    cubemap.unwrap_or(white_texture),
                    shadowmap[0],
                    shadowmap[1],
                    shadowmap[2],
                    shadowmap[3],
                ];
                ctx.apply_pipeline(&bindings.pipeline);
                ctx.apply_bindings_from_slice(
                    &bindings.vertex_buffers,
                    bindings.index_buffer,
                    &images,
                );

                let (proj, view) = camera.proj_view();
                //depth_view_proj = proj * view;

                let projection = proj * view;
                let time = (crate::time::get_time()) as f32;
                let time = glam::vec4(time, time.sin(), time.cos(), 0.);

                let model = transform * node.transform.matrix();
                let model_inverse = model.inverse();
                ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms {
                    projection,
                    shadow_projection: shadow_proj,
                    model,
                    model_inverse,
                    color: bindings.color,
                    shadow_cascades,
                    shadow_casters,
                    material: bindings.material,
                    camera_pos: camera.position,
                }));

                let buffer_size = ctx.buffer_size(bindings.index_buffer) as i32 / 2;
                ctx.draw(0, buffer_size, 1);
            }
        }

        // unsafe {
        //     use miniquad::gl;
        //     gl::glPolygonMode(gl::GL_FRONT_AND_BACK, gl::GL_FILL);
        // }
    }

    // pub fn set_transform(&self, model: usize, transform: Mat4) {
    //     self.models[model].1 = transform;
    // }

    pub fn draw(&mut self, camera: &Camera) {
        let _z = telemetry::ZoneGuard::new("Scene::draw");

        let clipping_planes = frustum::projection_planes(camera);
        let (proj, view) = camera.proj_view();
        let mut clear_action = PassAction::Nothing;
        {
            let _z = telemetry::ZoneGuard::new("environment");

            if let crate::camera::Environment::Skybox(ref cubemap) = camera.environment {
                cubemap.draw(&mut **self.quad_ctx.lock().unwrap(), &proj, &view);
            }

            if let crate::camera::Environment::Solid(color) = camera.environment {
                clear_action = PassAction::clear_color(color.r, color.g, color.b, color.a);
            }

            unsafe {
                miniquad::gl::glFlush();
                miniquad::gl::glFinish();
            }
        }
        let mut ctx = self.quad_ctx.lock().unwrap();

        let mut shadow_proj = Default::default();
        let mut cascade_clips = Default::default();
        let casters_count = self.shadow_casters.len();
        let mut split_count = 0;
        if let Some(shadow_caster) = self.shadow_casters.get(0) {
            split_count = match shadow_caster.split {
                ShadowSplit::Orthogonal => 1,
                ShadowSplit::PSSM2 => 2,
                ShadowSplit::PSSM4 => 4,
            };
            let _z = telemetry::ZoneGuard::new("shadows");
            (shadow_proj, cascade_clips) = self.shadowmap.draw_shadow_pass(
                ctx.as_mut(),
                &self.models[..],
                &camera,
                shadow_caster,
                clipping_planes,
            );

            unsafe {
                miniquad::gl::glFlush();
                miniquad::gl::glFinish();
            }
        }

        if let Some(pass) = camera.render_target.as_ref().map(|rt| rt.render_pass) {
            ctx.begin_pass(Some(pass), clear_action);
        } else {
            ctx.begin_default_pass(clear_action);
        }

        {
            let _z = telemetry::ZoneGuard::new("models");
            for model in &self.models {
                Scene::draw_model(
                    ctx.as_mut(),
                    self.white_texture,
                    self.black_texture,
                    model,
                    camera,
                    shadow_proj,
                    cascade_clips,
                    [
                        self.shadowmap.depth_img[0],
                        self.shadowmap.depth_img[1],
                        self.shadowmap.depth_img[2],
                        self.shadowmap.depth_img[3],
                    ],
                    [casters_count as _, split_count as _, 0, 0],
                    clipping_planes,
                );
            }
            unsafe {
                miniquad::gl::glFlush();
                miniquad::gl::glFinish();
            }
        }
        ctx.end_render_pass();
    }

    pub fn draw_shadow_debug(&mut self) {
        let mut ctx = self.quad_ctx.lock().unwrap();

        self.shadowmap
            .dbg
            .draw(ctx.as_mut(), &self.shadowmap.depth_img[..]);
    }
}

pub mod shader {
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
                "ShadowMap0".to_string(),
                "ShadowMap1".to_string(),
                "ShadowMap2".to_string(),
                "ShadowMap3".to_string(),
            ],
            uniforms: UniformBlockLayout {
                uniforms: vec![
                    UniformDesc::new("Projection", UniformType::Mat4),
                    UniformDesc::array(UniformDesc::new("ShadowProjection", UniformType::Mat4), 4),
                    UniformDesc::new("Model", UniformType::Mat4),
                    UniformDesc::new("ModelInverse", UniformType::Mat4),
                    UniformDesc::new("Color", UniformType::Float4),
                    UniformDesc::new("ShadowCascades", UniformType::Float4),
                    UniformDesc::new("ShadowCasters", UniformType::Int4),
                    UniformDesc::new("Material", UniformType::Float4),
                    UniformDesc::new("CameraPosition", UniformType::Float3),
                ],
            },
        }
    }

    #[repr(C)]
    pub struct Uniforms {
        pub projection: glam::Mat4,
        pub shadow_projection: [glam::Mat4; 4],
        pub model: glam::Mat4,
        pub model_inverse: glam::Mat4,
        pub color: [f32; 4],
        pub shadow_cascades: [f32; 4],
        pub shadow_casters: [i32; 4], // count, split, 0, 0
        pub material: [f32; 4],       // metallic, roughness, 0, 0,
        pub camera_pos: glam::Vec3,
    }
}
