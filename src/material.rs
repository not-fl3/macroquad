//! Custom materials - shaders, uniforms.

use crate::{get_context, texture::Texture2D, warn};

use miniquad::{PipelineParams, ShaderError, UniformType, *};

use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Uniform {
    name: String,
    uniform_type: UniformType,
    byte_offset: usize,
}

/// Material instance loaded on GPU.
#[derive(Clone, Debug)]
pub struct Material {
    pub(crate) pipeline_2d: miniquad::Pipeline,
    pub(crate) pipeline_3d: miniquad::Pipeline,
    pub(crate) wants_screen_texture: bool,
    pub(crate) uniforms: Vec<Uniform>,
    pub(crate) uniforms_data: Vec<u8>,
    pub(crate) textures: Vec<String>,
    pub(crate) textures_data: BTreeMap<String, TextureId>,
}

impl Material {
    // TODO
    pub(crate) fn new2(
        ctx: &mut dyn miniquad::RenderingBackend,
        shader: ShaderId,
        params: PipelineParams,
        mut uniforms: Vec<(String, UniformType)>,
        textures: Vec<String>,
    ) -> Result<Material, ShaderError> {
        // TODO!
        let wants_screen_texture = false;

        let pipeline_2d = ctx.new_pipeline_with_params(
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("position", VertexFormat::Float3),
                VertexAttribute::new("texcoord", VertexFormat::Float2),
                VertexAttribute::new("color0", VertexFormat::Byte4),
            ],
            shader,
            params,
        );

        let pipeline_3d = ctx.new_pipeline_with_params(
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
            params,
        );

        let mut max_offset = 0;

        for (name, kind) in shader::uniforms().into_iter().rev() {
            uniforms.insert(0, (name.to_owned(), kind));
        }

        let uniforms = uniforms
            .iter()
            .scan(0, |offset, uniform| {
                let uniform_byte_size = uniform.1.size();
                let uniform = Uniform {
                    name: uniform.0.clone(),
                    uniform_type: uniform.1,
                    byte_offset: *offset,
                };
                *offset += uniform_byte_size;
                max_offset = *offset;

                Some(uniform)
            })
            .collect();

        Ok(Material {
            pipeline_2d,
            pipeline_3d,
            wants_screen_texture,
            uniforms,
            uniforms_data: vec![0; max_offset],
            textures,
            textures_data: BTreeMap::new(),
        })
    }

    pub fn new(
        vertex_shader: &str,
        fragment_shader: &str,
        params: MaterialParams,
    ) -> Result<Material, ShaderError> {
        let ctx = &mut *get_context().quad_context;

        let shader = ctx
            .new_shader(
                ShaderSource {
                    glsl_vertex: Some(vertex_shader),
                    glsl_fragment: Some(fragment_shader),
                    metal_shader: None,
                },
                shader::meta(),
            )
            .unwrap();

        Self::new2(
            ctx,
            shader,
            params.pipeline_params,
            params.uniforms,
            params.textures,
        )
    }

    /// Set GPU uniform value for this material.
    /// "name" should be from "uniforms" list used for material creation.
    /// Otherwise uniform value would be silently ignored.
    pub fn set_uniform<T>(&mut self, name: &str, uniform: T) {
        let uniform_meta = self.uniforms.iter().find(
            |Uniform {
                 name: uniform_name, ..
             }| uniform_name == name,
        );
        if uniform_meta.is_none() {
            warn!("Trying to set non-existing uniform: {}", name);
            return;
        }
        let uniform_meta = uniform_meta.unwrap();
        let uniform_format = uniform_meta.uniform_type;
        let uniform_byte_size = uniform_format.size();
        let uniform_byte_offset = uniform_meta.byte_offset;

        if std::mem::size_of::<T>() != uniform_byte_size {
            warn!(
                "Trying to set uniform {} sized {} bytes value of {} bytes",
                name,
                std::mem::size_of::<T>(),
                uniform_byte_size
            );
            return;
        }
        macro_rules! transmute_uniform {
            ($uniform_size:expr, $byte_offset:expr, $n:expr) => {
                if $uniform_size == $n {
                    let data: [u8; $n] = unsafe { std::mem::transmute_copy(&uniform) };

                    for i in 0..$uniform_size {
                        self.uniforms_data[$byte_offset + i] = data[i];
                    }
                }
            };
        }
        transmute_uniform!(uniform_byte_size, uniform_byte_offset, 4);
        transmute_uniform!(uniform_byte_size, uniform_byte_offset, 8);
        transmute_uniform!(uniform_byte_size, uniform_byte_offset, 12);
        transmute_uniform!(uniform_byte_size, uniform_byte_offset, 16);
        transmute_uniform!(uniform_byte_size, uniform_byte_offset, 64);
    }

    pub fn set_texture(&mut self, name: &str, texture: Texture2D) {
        self.textures
            .iter()
            .find(|x| *x == name)
            .unwrap_or_else(|| {
                panic!(
                    "can't find texture with name '{}', there are only this names: {:?}",
                    name, self.textures
                )
            });

        // *self
        //     .textures_data
        //     .entry(name.to_owned())
        //     .or_insert(texture.texture) = texture.texture;
        unimplemented!()
    }

    /// Delete this material. Using deleted material for either rendering
    /// or uniforms manipulation will result internal GL errors.
    pub fn delete(&mut self) {
        //get_context().gl.delete_pipeline(self.pipeline);
    }
}

/// Params used for material loading.
/// It is not possible to change material params at runtime, so this
/// struct is used only once - at "load_material".
pub struct MaterialParams {
    /// miniquad pipeline configuration for this material.
    /// Things like blending, culling, depth dest
    pub pipeline_params: PipelineParams,

    /// List of custom uniforms used in this material
    pub uniforms: Vec<(String, UniformType)>,

    /// List of textures used in this material
    pub textures: Vec<String>,
}

impl Default for MaterialParams {
    fn default() -> Self {
        MaterialParams {
            pipeline_params: Default::default(),
            uniforms: vec![],
            textures: vec!["Texture".to_string()],
        }
    }
}

// pub fn load_material(
//     shader: crate::ShaderSource,
//     params: MaterialParams,
// ) -> Result<Material, Error> {
//     let context = &mut get_context();

//     let pipeline = context.gl.make_pipeline(
//         &mut *context.quad_context,
//         shader,
//         params.pipeline_params,
//         params.uniforms,
//         params.textures,
//     )?;

//     Ok(Material {
//         pipeline: Arc::new(GlPipelineGuarded(pipeline)),
//     })
// }

// /// All following macroquad rendering calls will use the given material.
// pub fn gl_use_material(material: &Material) {
//     get_context().gl.pipeline(Some(material.pipeline.0));
// }

/// Use default macroquad material.
pub fn gl_use_default_material() {
    //get_context().gl.pipeline(None);
    unimplemented!()
}

#[doc(hidden)]
pub mod shaders {
    type IncludeFilename = String;
    type IncludeContent = String;

    #[derive(Debug, Clone)]
    pub struct PreprocessorConfig {
        pub includes: Vec<(IncludeFilename, IncludeContent)>,
    }
    impl Default for PreprocessorConfig {
        fn default() -> PreprocessorConfig {
            PreprocessorConfig { includes: vec![] }
        }
    }

    impl PreprocessorConfig {}

    pub fn preprocess_shader(source: &str, config: &PreprocessorConfig) -> String {
        let mut res = source.chars().collect::<Vec<_>>();

        fn find(data: &[char], n: &mut usize, target: &str) -> bool {
            if *n >= data.len() {
                return false;
            }
            let target = target.chars().collect::<Vec<_>>();

            'outer: for i in *n..data.len() {
                for j in 0..target.len() {
                    if data[i + j] != target[j] {
                        *n += 1;
                        continue 'outer;
                    }
                }
                return true;
            }
            false
        }

        fn skip_character(data: &[char], n: &mut usize, target: char) {
            while *n < data.len() && data[*n] == target {
                *n += 1;
            }
        }

        let mut i = 0;
        while find(&res, &mut i, "#include") {
            let directive_start_ix = i;
            i += "#include".len();
            skip_character(&res, &mut i, ' ');
            assert!(res[i] == '\"');
            i += 1;
            let filename_start_ix = i;
            find(&res, &mut i, "\"");
            let filename_end_ix = i;
            let filename = res[filename_start_ix..filename_end_ix]
                .iter()
                .cloned()
                .collect::<String>();

            let include_content = config
                .includes
                .iter()
                .find(|(name, _)| name == &filename)
                .expect(&format!(
                    "Include file {} in not on \"includes\" list",
                    filename
                ));

            let _ = res
                .splice(
                    directive_start_ix..filename_end_ix + 1,
                    include_content.1.chars(),
                )
                .collect::<Vec<_>>();
        }

        res.into_iter().collect()
    }

    #[test]
    fn preprocessor_test() {
        let shader_string = r#"
#version blah blah

asd
asd

#include "hello.glsl"

qwe
"#;

        let preprocessed = r#"
#version blah blah

asd
asd

iii
jjj

qwe
"#;

        let result = preprocess_shader(
            shader_string,
            &PreprocessorConfig {
                includes: vec![("hello.glsl".to_string(), "iii\njjj".to_string())],
                ..Default::default()
            },
        );

        assert_eq!(result, preprocessed);
    }
}

mod shader {
    use miniquad::{ShaderMeta, UniformBlockLayout, UniformDesc, UniformType};

    pub const VERTEX: &str = r#"#version 100
    attribute vec3 position;
    attribute vec2 texcoord;
    attribute vec4 color0;

    varying lowp vec2 uv;
    varying lowp vec4 color;

    uniform mat4 Model;
    uniform mat4 Projection;

    void main() {
        gl_Position = Projection * Model * vec4(position, 1);
        color = color0 / 255.0;
        uv = texcoord;
    }"#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec4 color;
    varying lowp vec2 uv;

    uniform sampler2D Texture;

    void main() {
        gl_FragColor = color * texture2D(Texture, uv) ;
    }"#;

    pub fn uniforms() -> Vec<(&'static str, UniformType)> {
        vec![
            ("Projection", UniformType::Mat4),
            ("Model", UniformType::Mat4),
            ("_Time", UniformType::Float4),
        ]
    }

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["Texture".to_string(), "_ScreenTexture".to_string()],
            uniforms: UniformBlockLayout {
                uniforms: uniforms()
                    .into_iter()
                    .map(|(name, kind)| UniformDesc::new(name, kind))
                    .collect(),
            },
        }
    }
}
