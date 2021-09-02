//! Custom materials - shaders, uniforms.

use crate::get_context;
use crate::prelude::Texture2D;
use crate::quad_gl::GlPipeline;
use miniquad::{PipelineParams, ShaderError, UniformType};

/// Material instance loaded on GPU.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Material {
    pipeline: GlPipeline,
}

impl Material {
    /// Set GPU uniform value for this material.
    /// "name" should be from "uniforms" list used for material creation.
    /// Otherwise uniform value would be silently ignored.
    pub fn set_uniform<T>(&self, name: &str, uniform: T) {
        get_context().gl.set_uniform(self.pipeline, name, uniform);
    }

    pub fn set_texture(&self, name: &str, texture: Texture2D) {
        get_context().gl.set_texture(self.pipeline, name, texture);
    }

    /// Delete this material. Using deleted material for either rendering
    /// or uniforms manipulation will result internal GL errors.
    pub fn delete(&mut self) {
        get_context().gl.delete_pipeline(self.pipeline);
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
            textures: vec![],
        }
    }
}

pub fn load_material(
    vertex_shader: &str,
    fragment_shader: &str,
    params: MaterialParams,
) -> Result<Material, ShaderError> {
    let context = &mut get_context();

    let pipeline = context.gl.make_pipeline(
        &mut context.quad_context,
        vertex_shader,
        fragment_shader,
        params.pipeline_params,
        params.uniforms,
        params.textures,
    )?;

    Ok(Material { pipeline })
}

/// All following macroquad rendering calls will use the given material.
pub fn gl_use_material(material: Material) {
    get_context().gl.pipeline(Some(material.pipeline));
}

/// Use default macroquad material.
pub fn gl_use_default_material() {
    get_context().gl.pipeline(None);
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
