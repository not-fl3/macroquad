//! Custom materials - shaders, uniforms. 

use quad_gl::GlPipeline;
use crate::get_context;
use miniquad::{PipelineParams, UniformType, ShaderError};

/// Material instance loaded on GPU. 
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Material {
    pipeline: GlPipeline,
}

impl Material {
    /// Set GPU uniform valuye for this material.
    /// "name" should be from "uniforms" list used for material creation.
    /// Otherwise uniform value would be silently ignored.
    pub fn set_uniform<T>(&self, name: &str, uniform: T) {
        let context = &mut get_context().draw_context;

        context.gl.set_uniform(self.pipeline, name, uniform);
    }

    /// Delete this material. Using deleted material for either rendering
    /// or uniforms manipulation will result internal GL errors.
    pub fn delete(&mut self) {
        let context = &mut get_context().draw_context;

        context.gl.delete_pipeline(self.pipeline);
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
}

impl Default for MaterialParams {
    fn default() -> Self {
        MaterialParams {
            pipeline_params: Default::default(),
            uniforms: vec![]
        }
    }
}

pub fn load_material(
    vertex_shader: &str,
    fragment_shader: &str,
    params: MaterialParams,
) -> Result<Material, ShaderError> {
    let context = &mut get_context();

    let pipeline = context.draw_context.gl.make_pipeline(
        &mut context.quad_context,
        vertex_shader,
        fragment_shader,
        params.pipeline_params,
        params.uniforms,
    )?;

    Ok(Material { pipeline })
}

/// Al followed macroquad rendering calls will use the given material.
pub fn gl_use_material(material: Material) {
    let context = &mut get_context().draw_context;

    context.gl.pipeline(Some(material.pipeline));
}

/// Use default macroquad material. 
pub fn gl_use_default_material() {
    let context = &mut get_context().draw_context;

    context.gl.pipeline(None);
}
