use crate::file::{load_file, load_string};

use miniquad::{
    BlendFactor, BlendState, BlendValue, BufferLayout, BufferSource, BufferType, BufferUsage,
    Comparison, Equation, MipmapFilterMode, PipelineParams, TextureWrap, VertexAttribute,
    VertexFormat, VertexStep,
};
use quad_gl::{
    image,
    math::{vec3, Quat, Vec3},
    scene::{shader, Material2, Model, Node, NodeData, Transform, AABB},
    texture::FilterMode,
    Error,
};

use std::sync::{Arc, Mutex};

pub struct Resources {
    quad_ctx: Arc<Mutex<Box<miniquad::Context>>>,
    quad_gl: Arc<Mutex<quad_gl::QuadGl>>,
}

impl Resources {
    pub fn new(
        quad_ctx: Arc<Mutex<Box<miniquad::Context>>>,
        quad_gl: Arc<Mutex<quad_gl::QuadGl>>,
    ) -> Resources {
        Resources { quad_ctx, quad_gl }
    }

    pub async fn load_gltf(&self, path: &str) -> Result<Model, crate::Error> {
        use nanogltf::{utils, Gltf};
        use std::borrow::Cow;

        let mut ctx = self.quad_ctx.lock().unwrap();

        let gltf = load_string(path).await?;
        let gltf = Gltf::from_json(&gltf).unwrap();
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
                    load_file(&format!("{}/{}", parent, uri)).await?
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
                    Cow::from(load_file(&format!("{}/{}", parent, uri)).await?)
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
            textures.push(quad_gl::texture::Texture2D::from_miniquad_texture(texture));
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
            let mut materials = Vec::new();

            for primitive in &mesh.primitives {
                let material = &gltf.materials[primitive.material.unwrap()];
                let color = material.pbr_metallic_roughness.base_color_factor;

                let base_color_texture = &material.pbr_metallic_roughness.base_color_texture;
                let base_color_texture = base_color_texture
                    .as_ref()
                    .map(|t| textures[t.index].clone());
                let metallic_roughness_texture = material
                    .pbr_metallic_roughness
                    .metallic_roughness_texture
                    .as_ref()
                    .map(|t| textures[t.index].clone());
                let emissive_texture = material
                    .emissive_texture
                    .as_ref()
                    .map(|t| textures[t.index].clone());
                let occlusion_texture = material
                    .occlusion_texture
                    .as_ref()
                    .map(|t| textures[t.index].clone());
                let normal_texture = material
                    .normal_texture
                    .as_ref()
                    .map(|t| textures[t.index].clone());
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
                let instancing = vec![vec3(0.0, 0.0, 0.0)];
                let instancing_buffer =
                    ctx.new_buffer(BufferType::VertexBuffer, BufferUsage::Immutable, unsafe {
                        BufferSource::slice(&instancing[..])
                    });

                let shader = quad_gl::scene::Shader::default(ctx.as_mut());

                bindings.push(NodeData {
                    vertex_buffers: vec![
                        vertex_buffer,
                        uvs_buffer,
                        normals_buffer,
                        instancing_buffer,
                    ],
                    index_buffer,
                });
                materials.push(Material2 {
                    color,
                    base_color_texture,
                    emissive_texture,
                    normal_texture,
                    occlusion_texture,
                    metallic_roughness_texture,
                    metallic: material.pbr_metallic_roughness.metallic_factor as f32,
                    roughness: material.pbr_metallic_roughness.roughness_factor as f32,
                    shader,
                });
            }

            nodes.push(Node {
                name: node
                    .name
                    .clone()
                    .unwrap_or("unnamed".to_string())
                    .to_owned(),
                data: bindings,
                materials,
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
    ) -> Result<quad_gl::cubemap::Cubemap, crate::Error> {
        let cubemap = [
            &load_file(texture_px).await?[..],
            &load_file(texture_nx).await?[..],
            &load_file(texture_py).await?[..],
            &load_file(texture_ny).await?[..],
            &load_file(texture_pz).await?[..],
            &load_file(texture_nz).await?[..],
        ];
        let mut quad_ctx = self.quad_ctx.lock().unwrap();
        let cubemap = quad_gl::cubemap::Cubemap::new(quad_ctx.as_mut(), &cubemap[..]);
        Ok(cubemap)
    }

    pub async fn load_texture(
        &self,
        path: &str,
    ) -> Result<quad_gl::texture::Texture2D, crate::Error> {
        let bytes = &load_file(path).await?;
        Ok(self.quad_gl.lock().unwrap().load_texture(&bytes))
    }
}
