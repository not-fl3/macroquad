use crate::get_context;

// somewhat hacky workaround to keep everything else working
pub struct SceneHandler {
    pub update: fn(),
    pub in_fixed_update: fn() -> bool,
    pub fixed_frame_time: fn() -> f32,
    pub allocated_memory: fn() -> usize,
}

pub fn register_handler(handler: SceneHandler) {
    let context = get_context();
    context.scene_handler = Some(handler);
}
