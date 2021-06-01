use crate::time::get_time;

use std::collections::HashMap;

static mut PROFILER: Option<Profiler> = None;

fn get_profiler() -> &'static mut Profiler {
    unsafe {
        PROFILER.get_or_insert_with(|| Profiler {
            frame: Frame::new(),
            queries: HashMap::new(),
            active_query: None,
            prev_frame: Frame::new(),
            enabled: false,
            enable_request: None,
            capture_request: false,
            capture: false,
            drawcalls: vec![],
            strings: vec![],
        })
    }
}

#[derive(Debug, Clone)]
pub struct Zone {
    pub name: String,
    pub start_time: f64,
    pub duration: f64,
    pub children: Vec<Zone>,

    parent: *mut Zone,
}

impl Zone {
    fn clone(&self, parent: *mut Zone) -> Zone {
        Zone {
            name: self.name.clone(),
            start_time: self.start_time,
            duration: self.duration,
            children: self
                .children
                .iter()
                .map(|zone| zone.clone(self as *const _ as *mut _))
                .collect(),
            parent,
        }
    }
}

pub struct ZoneGuard {
    _marker: (),
}

impl ZoneGuard {
    pub fn new(name: &str) -> ZoneGuard {
        begin_zone(name);
        ZoneGuard { _marker: () }
    }
}

impl Drop for ZoneGuard {
    fn drop(&mut self) {
        end_zone();
    }
}

pub fn enable() {
    get_profiler().enable_request = Some(true);
}

pub fn disable() {
    get_profiler().enable_request = Some(false);
}

pub fn begin_zone(name: &str) {
    if get_profiler().enabled {
        get_profiler().begin_zone(name);
    }
}

pub fn end_zone() {
    if get_profiler().enabled {
        get_profiler().end_zone();
    }
}

pub fn begin_gpu_query(name: &str) {
    get_profiler().begin_gpu_query(name);
}

pub fn end_gpu_query() {
    get_profiler().end_gpu_query();
}

/// Workaround to stop gl capture on debug rendering
#[doc(hidden)]
pub fn pause_gl_capture() {
    if get_profiler().capture {
        crate::get_context().gl.capture(false);
    }
}

/// Workaround to stop gl capture on debug rendering
pub fn resume_gl_capture() {
    if get_profiler().capture {
        crate::get_context().gl.capture(false);
    }
}

pub(crate) fn reset() {
    let profiler = get_profiler();

    assert!(
        get_profiler().frame.active_zone.is_null(),
        "New frame started with unpaired begin/end zones."
    );

    profiler.frame.full_frame_time = crate::time::get_frame_time();

    std::mem::swap(&mut profiler.prev_frame, &mut profiler.frame);
    profiler.frame = Frame::new();

    if let Some(enable) = profiler.enable_request.take() {
        profiler.enabled = enable;
    }

    if profiler.capture {
        profiler.capture = false;
        crate::get_context().gl.capture(false);
    }

    if profiler.capture_request {
        profiler.drawcalls.clear();
        profiler.capture = true;
        crate::get_context().gl.capture(true);
        profiler.capture_request = false;
    }
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub full_frame_time: f32,
    pub zones: Vec<Zone>,
    active_zone: *mut Zone,
}

impl Frame {
    fn new() -> Frame {
        Frame {
            full_frame_time: 0.0,
            zones: vec![],
            active_zone: std::ptr::null_mut(),
        }
    }

    pub fn try_clone(&self) -> Option<Frame> {
        if self.active_zone.is_null() == false {
            return None;
        }

        Some(Frame {
            full_frame_time: self.full_frame_time,
            zones: self
                .zones
                .iter()
                .map(|zone| zone.clone(std::ptr::null_mut()))
                .collect(),
            active_zone: std::ptr::null_mut(),
        })
    }
}

pub fn frame() -> Frame {
    get_profiler().prev_frame.clone()
}

pub fn gpu_queries() -> Vec<(String, u64)> {
    get_profiler()
        .queries
        .iter()
        .map(|(name, query)| (name.to_owned(), query.value))
        .collect()
}

pub fn sample_gpu_queries() {
    for query in &mut get_profiler().queries {
        query.1.force_resume = true;
    }
}

struct Profiler {
    frame: Frame,
    prev_frame: Frame,
    queries: HashMap<String, GpuQuery>,
    active_query: Option<String>,
    enabled: bool,
    capture_request: bool,
    capture: bool,
    enable_request: Option<bool>,
    drawcalls: Vec<DrawCallTelemetry>,
    strings: Vec<String>,
}

impl Profiler {
    fn begin_gpu_query(&mut self, name: &str) {
        assert!(
            self.active_query.is_none(),
            "Only one active query is allowed by OpenGL"
        );

        let name = name.to_string();
        let query = self
            .queries
            .entry(name.clone())
            .or_insert_with(|| GpuQuery {
                query: miniquad::graphics::ElapsedQuery::new(),
                in_progress: false,
                value: 0,
                force_resume: false,
            });
        self.active_query = Some(name);
        if query.force_resume {
            query.in_progress = true;
            query.query.begin_query();
        }
    }

    fn end_gpu_query(&mut self) {
        let name = self
            .active_query
            .take()
            .expect("End query without begin query");
        let mut query = self.queries.get_mut(&name).unwrap();
        if query.in_progress {
            query.force_resume = false;
            query.in_progress = false;
            query.query.end_query();
        }
        if query.query.is_available() {
            query.value = query.query.get_result();
        }
    }

    fn begin_zone(&mut self, name: &str) {
        let zones = if self.frame.active_zone.is_null() {
            &mut self.frame.zones
        } else {
            unsafe { &mut (&mut *self.frame.active_zone).children }
        };

        zones.push(Zone {
            name: name.to_string(),
            start_time: get_time(),
            duration: 0.0,
            parent: self.frame.active_zone,
            children: vec![],
        });
        self.frame.active_zone = zones.last_mut().unwrap() as _;
    }

    fn end_zone(&mut self) {
        assert!(
            self.frame.active_zone.is_null() == false,
            "end_zone called without begin_zone"
        );

        let start_time = unsafe { (&mut *self.frame.active_zone).start_time };
        let duration = get_time() - start_time;

        unsafe { (&mut *self.frame.active_zone).duration = duration };
        self.frame.active_zone = unsafe { (&mut *self.frame.active_zone).parent };
    }
}

pub struct GpuQuery {
    pub query: miniquad::graphics::ElapsedQuery,
    pub in_progress: bool,
    pub value: u64,
    pub force_resume: bool,
}

pub fn scene_allocated_memory() -> usize {
    use crate::experimental::scene;

    scene::allocated_memory()
}

/// ```skip
/// {
///    let _t = telemetry::LogTimeGuard::new("Atlas build time");
///     mq::texture::build_textures_atlas();
/// }
/// ```
/// Will add "Time query: Atlas build time, 0.5s" string to
/// `telemetry::strings()`
pub struct LogTimeGuard<'a> {
    name: &'a str,
    start_time: f64,
}

impl<'a> LogTimeGuard<'a> {
    pub fn new(name: &'a str) -> LogTimeGuard {
        LogTimeGuard {
            name,
            start_time: get_time(),
        }
    }
}

impl<'a> Drop for LogTimeGuard<'a> {
    fn drop(&mut self) {
        log_string(&format!(
            "Time query: {}, {:.1}s",
            self.name,
            get_time() - self.start_time
        ))
    }
}

pub fn log_string(string: &str) {
    get_profiler().strings.push(string.to_owned());
}

pub fn drawcalls() -> Vec<DrawCallTelemetry> {
    get_profiler().drawcalls.clone()
}

pub fn strings() -> Vec<String> {
    get_profiler().strings.clone()
}

pub fn capture_frame() {
    get_profiler().capture_request = true;
}

#[derive(Clone, Debug)]
pub struct DrawCallTelemetry {
    pub indices_count: usize,
    pub texture: miniquad::Texture,
}

pub(crate) fn track_drawcall(
    pipeline: &miniquad::Pipeline,
    bindings: &miniquad::Bindings,
    indices_count: usize,
) {
    let ctx = crate::get_context();
    let texture = miniquad::Texture::new_render_texture(
        &mut ctx.quad_context,
        miniquad::TextureParams {
            width: 128,
            height: 128,
            ..Default::default()
        },
    );

    let pass = Some(miniquad::RenderPass::new(
        &mut ctx.quad_context,
        texture,
        None,
    ));
    ctx.quad_context
        .begin_pass(pass, miniquad::PassAction::clear_color(0.4, 0.8, 0.4, 1.));
    ctx.quad_context.apply_pipeline(pipeline);
    ctx.quad_context.apply_bindings(bindings);
    ctx.quad_context.draw(0, indices_count as _, 1);
    ctx.quad_context.end_render_pass();

    get_profiler().drawcalls.push(DrawCallTelemetry {
        indices_count,
        texture,
    });
}
