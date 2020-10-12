use macroquad::prelude::*;
use macroquad::window::miniquad::*;

#[derive(Debug, Clone)]
pub struct EmitterConfig {
    /// If false - particles spawns at position supplied to .draw(), but afterwards lives in current camera coordinate system.
    /// If false particles use coordinate system originated to the emitter draw position
    pub local_coords: bool,
    /// If true only one emission cycle occures. May be re-emitted by .emit() call.
    pub one_shot: bool,
    /// Lifespan of individual particle.
    pub lifetime: f32,
    /// Particle lifetime randomness ratio.
    /// Each particle will spawned with "lifetime = lifetime - lifetime * rand::gen_range(0.0, lifetime_randomness)".
    pub lifetime_randomness: f32,
    /// 0..1 value, how rapidly particles in emission cycle are emitted.
    /// With 0 particles will be emitted with equal gap.
    /// With 1 all the particles will be emitted at the beginning of the cycle.
    pub explosiveness: f32,
    /// Amount of particles emitted in one emission cycle.
    pub amount: u32,
    /// Particles are emitting when "emitting" is true.
    /// If its a "oneshot" emitter, emitting will switch to false after active emission cycle.
    pub emitting: bool,
    /// Unit vector specifying emission direction.
    pub initial_direction: Vec2,
    /// Angle from 0 to "2 * Pi" for random fluctuation for direction vector.
    pub initial_direction_spread: f32,
    /// Initial speed for each emitted particle.
    /// Direction of the initial speed vector is affected by "direction" and "spread"
    pub initial_velocity: f32,
    /// Each particle is a square "initial_size x initial_size" size.
    pub initial_size: f32,
    /// Particles rendering mode.
    pub blend_mode: BlendMode,

    /// Gravity applied to each individual particle.
    pub gravity: Vec2,

    /// Particle texture. If none particles going to be white squares.
    pub texture: Option<Texture2D>,

    /// For animated texture specify spritesheet layout.
    /// If none the whole texture will be used.
    pub atlas: Option<AtlasConfig>,
}

#[derive(Debug, Clone, Copy)]
pub enum BlendMode {
    /// Colors of overlapped particles will be blended by alpha channel.
    Alpha,
    /// Colors of overlapped particles will be added to each other.
    Additive,
}

#[derive(Debug, Clone)]
pub struct AtlasConfig {
    n: u16,
    m: u16,
    start_index: u16,
    end_index: u16,
}

impl AtlasConfig {
    pub fn new<T: std::ops::RangeBounds<u16>>(n: u16, m: u16, range: T) -> AtlasConfig {
        let start_index = match range.start_bound() {
            std::ops::Bound::Unbounded => 0,
            std::ops::Bound::Included(i) => *i,
            std::ops::Bound::Excluded(i) => i + 1,
        };
        let end_index = match range.end_bound() {
            std::ops::Bound::Unbounded => n * m,
            std::ops::Bound::Included(i) => i - 1,
            std::ops::Bound::Excluded(i) => *i,
        };

        AtlasConfig {
            n,
            m,
            start_index,
            end_index,
        }
    }
}

impl Default for EmitterConfig {
    fn default() -> EmitterConfig {
        EmitterConfig {
            local_coords: false,
            one_shot: false,
            lifetime: 1.0,
            lifetime_randomness: 0.0,
            amount: 8,
            explosiveness: 0.0,
            emitting: true,
            initial_direction: vec2(0., -1.),
            initial_direction_spread: 0.,
            initial_velocity: 200.0,
            initial_size: 10.0,
            blend_mode: BlendMode::Alpha,
            gravity: vec2(0.0, 0.0),
            texture: None,
            atlas: None,
        }
    }
}

#[repr(C)]
struct GpuParticle {
    pos: Vec3,
    uv: Vec4,
}

struct CpuParticle {
    velocity: Vec2,
    lived: f32,
    lifetime: f32,
    frame: u16,
}

pub struct Emitter {
    pipeline: Pipeline,
    bindings: Bindings,

    gpu_particles: Vec<GpuParticle>,
    cpu_counterpart: Vec<CpuParticle>,

    last_emit_time: f32,
    time_passed: f32,

    particles_spawned: u64,
    position: Vec2,
    pub config: EmitterConfig,
}

impl Emitter {
    const MAX_PARTICLES: usize = 512 * 1024;

    pub fn new(config: EmitterConfig) -> Emitter {
        let InternalGlContext {
            quad_context: ctx, ..
        } = unsafe { get_internal_gl() };

        let r = config.initial_size;
        #[rustfmt::skip]
        let vertices: &[f32] = &[
            // positions    uv         colors
             -r, -r, 0.0,   0.0, 0.0,  1.0, 1.0, 1.0, 1.0,
              r, -r, 0.0,   1.0, 0.0,  1.0, 1.0, 1.0, 1.0,
              r,  r, 0.0,   1.0, 1.0,  1.0, 1.0, 1.0, 1.0,
             -r,  r, 0.0,   0.0, 1.0,  1.0, 1.0, 1.0, 1.0,
        ];
        // vertex buffer for static geometry
        let geometry_vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices);

        #[rustfmt::skip]
        let indices: &[u16] = &[
            0, 1, 2, 0, 2, 3
        ];
        let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &indices);

        // empty, dynamic instance-data vertex buffer
        let positions_vertex_buffer = Buffer::stream(
            ctx,
            BufferType::VertexBuffer,
            Self::MAX_PARTICLES * std::mem::size_of::<Vec3>(),
        );

        let bindings = Bindings {
            vertex_buffers: vec![geometry_vertex_buffer, positions_vertex_buffer],
            index_buffer,
            images: vec![config.texture.map_or_else(
                || Texture::from_rgba8(ctx, 1, 1, &[255, 255, 255, 255]),
                |texture| texture.raw_miniquad_texture_handle(),
            )],
        };

        let shader = Shader::new(ctx, shader::VERTEX, shader::FRAGMENT, shader::meta()).unwrap();

        let blend_mode = match config.blend_mode {
            BlendMode::Alpha => BlendState::new(
                Equation::Add,
                BlendFactor::Value(BlendValue::SourceAlpha),
                BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
            ),
            BlendMode::Additive => BlendState::new(
                Equation::Add,
                BlendFactor::Value(BlendValue::SourceAlpha),
                BlendFactor::One,
            ),
        };
        let pipeline = Pipeline::with_params(
            ctx,
            &[
                BufferLayout::default(),
                BufferLayout {
                    step_func: VertexStep::PerInstance,
                    ..Default::default()
                },
            ],
            &[
                VertexAttribute::with_buffer("pos", VertexFormat::Float3, 0),
                VertexAttribute::with_buffer("uv", VertexFormat::Float2, 0),
                VertexAttribute::with_buffer("color0", VertexFormat::Float4, 0),
                VertexAttribute::with_buffer("inst_pos", VertexFormat::Float3, 1),
                VertexAttribute::with_buffer("inst_uv", VertexFormat::Float4, 1),
            ],
            shader,
            PipelineParams {
                color_blend: Some(blend_mode),
                ..Default::default()
            },
        );

        Emitter {
            config,
            pipeline,
            bindings,
            position: vec2(0.0, 0.0),
            gpu_particles: Vec::with_capacity(Self::MAX_PARTICLES),
            cpu_counterpart: Vec::with_capacity(Self::MAX_PARTICLES),
            particles_spawned: 0,
            last_emit_time: 0.0,
            time_passed: 0.0,
        }
    }

    fn emit_particle(&mut self) {
        fn random_initial_vector(dir: Vec2, spread: f32, velocity: f32) -> Vec2 {
            let angle = rand::gen_range(-spread / 2.0, spread / 2.0);

            let quat = glam::Quat::from_rotation_z(angle);
            let dir = quat * vec3(dir.x(), dir.y(), 0.0);
            let res = dir * velocity;

            vec2(res.x(), res.y())
        }

        let particle = if self.config.local_coords {
            GpuParticle {
                pos: vec3(0.0, 0.0, 0.0),
                uv: vec4(1.0, 1.0, 0.0, 0.0),
            }
        } else {
            GpuParticle {
                pos: vec3(self.position.x(), self.position.y(), 0.0),
                uv: vec4(1.0, 1.0, 0.0, 0.0),
            }
        };

        self.particles_spawned += 1;
        self.gpu_particles.push(particle);
        self.cpu_counterpart.push(CpuParticle {
            velocity: random_initial_vector(
                vec2(
                    self.config.initial_direction.x(),
                    self.config.initial_direction.y(),
                ),
                self.config.initial_direction_spread,
                self.config.initial_velocity,
            ),
            lived: 0.0,
            lifetime: self.config.lifetime
                - self.config.lifetime * rand::gen_range(0.0, self.config.lifetime_randomness),
            frame: 0,
        });
    }

    fn update(&mut self, ctx: &mut Context, dt: f32) {
        if self.config.emitting {
            self.time_passed += dt;

            let gap = (self.config.lifetime / self.config.amount as f32)
                * (1.0 - self.config.explosiveness);

            let spawn_amount = if gap < 0.001 {
                // to prevent division by 0 problems
                self.config.amount as usize
            } else {
                // how many particles fits into this delta time
                ((self.time_passed - self.last_emit_time) / gap) as usize
            };

            for _ in 0..spawn_amount {
                self.last_emit_time = self.time_passed;

                if self.particles_spawned < self.config.amount as u64 {
                    self.emit_particle();
                } else if self.config.one_shot {
                    self.config.emitting = false;
                    self.particles_spawned = 0;
                }

                if self.gpu_particles.len() >= self.config.amount as usize {
                    break;
                }
            }
        }

        for (gpu, cpu) in self.gpu_particles.iter_mut().zip(&mut self.cpu_counterpart) {
            gpu.pos += vec3(cpu.velocity.x(), cpu.velocity.y(), 0.0) * dt;
            //cpu.velocity -= vec3(0.0, -10.0, 0.0);
            cpu.lived += dt;

            cpu.velocity += self.config.gravity * dt;

            if let Some(atlas) = &self.config.atlas {
                cpu.frame = (cpu.lived / cpu.lifetime
                    * (atlas.end_index - atlas.start_index) as f32)
                    as u16
                    + atlas.start_index;

                let x = cpu.frame % atlas.n;
                let y = cpu.frame / atlas.m;

                gpu.uv = vec4(
                    x as f32 / atlas.n as f32,
                    y as f32 / atlas.m as f32,
                    1.0 / atlas.n as f32,
                    1.0 / atlas.m as f32,
                );
            }
        }

        for i in (0..self.gpu_particles.len()).rev() {
            if self.cpu_counterpart[i].lived > self.cpu_counterpart[i].lifetime {
                self.gpu_particles.remove(i);
                self.cpu_counterpart.remove(i);

                if self.config.one_shot == false {
                    self.particles_spawned -= 1;
                }
            }
        }

        self.bindings.vertex_buffers[1].update(ctx, &self.gpu_particles[..]);
    }

    pub fn draw(&mut self, pos: Vec2) {
        self.position = pos;

        let mut gl = unsafe { get_internal_gl() };

        gl.flush();

        let InternalGlContext {
            quad_context: ctx,
            quad_gl,
        } = gl;

        let pass = quad_gl.get_active_render_pass();

        self.update(ctx, get_frame_time());

        if let Some(pass) = pass {
            ctx.begin_pass(pass, PassAction::Nothing);
        } else {
            ctx.begin_default_pass(PassAction::Nothing);
        }

        ctx.apply_pipeline(&self.pipeline);
        ctx.apply_bindings(&self.bindings);
        ctx.apply_uniforms(&shader::Uniforms {
            mvp: quad_gl.get_projection_matrix(),
            emitter_position: vec3(self.position.x(), self.position.y(), 0.0),
            local_coords: if self.config.local_coords { 1.0 } else { 0.0 },
        });
        ctx.draw(0, 6, self.gpu_particles.len() as i32);
        ctx.end_render_pass();
    }
}

mod shader {
    use super::*;

    pub const VERTEX: &str = r#"#version 100
    attribute vec3 pos;
    attribute vec2 uv;
    attribute vec4 color0;
    attribute vec3 inst_pos;
    attribute vec4 inst_uv;

    varying lowp vec4 color;
    varying lowp vec2 texcoord;

    uniform mat4 mvp;
    uniform float local_coords;
    uniform vec3 emitter_position;

    void main() {
        vec4 transformed = vec4(0.0, 0.0, 0.0, 0.0);

        if (local_coords == 0.0) {
           transformed = vec4(pos + inst_pos.xyz, 1.0);
        } else {
           transformed = vec4(pos + inst_pos.xyz + emitter_position.xyz, 1.0);
        }
        gl_Position =  mvp * transformed;
        color = color0;
        texcoord = uv * inst_uv.zw + inst_uv.xy;
    }
    "#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec2 texcoord;
    varying lowp vec4 color;
    uniform sampler2D texture;
    
    void main() {
        gl_FragColor = texture2D(texture, texcoord) * color;
    }
    "#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["texture".to_string()],
            uniforms: UniformBlockLayout {
                uniforms: vec![
                    UniformDesc::new("mvp", UniformType::Mat4),
                    UniformDesc::new("local_coords", UniformType::Float1),
                    UniformDesc::new("emitter_position", UniformType::Float3),
                ],
            },
        }
    }

    #[repr(C)]
    pub struct Uniforms {
        pub mvp: Mat4,
        pub local_coords: f32,
        pub emitter_position: Vec3,
    }
}
