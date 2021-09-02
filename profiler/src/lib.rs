use macroquad::{experimental::collections::storage, telemetry};

use macroquad::prelude::*;

use macroquad::ui::{hash, root_ui, widgets::Window, Ui};

pub struct ProfilerState {
    fps_buffer: Vec<f32>,
    frames_buffer: Vec<telemetry::Frame>,
    selected_frame: Option<telemetry::Frame>,
    profiler_window_opened: bool,
    paused: bool,
}

pub struct ProfilerParams {
    pub fps_counter_pos: Vec2,
}

impl Default for ProfilerParams {
    fn default() -> ProfilerParams {
        ProfilerParams {
            fps_counter_pos: vec2(10., 10.),
        }
    }
}

const FPS_BUFFER_CAPACITY: usize = 100;
const FRAMES_BUFFER_CAPACITY: usize = 400;

fn profiler_window(ui: &mut Ui, state: &mut ProfilerState) {
    fn zone_ui(ui: &mut Ui, zone: &telemetry::Zone, n: usize) {
        let label = format!(
            "{}: {:.4}ms {:.1}(1/t)",
            zone.name,
            zone.duration,
            1.0 / zone.duration
        );
        if zone.children.len() != 0 {
            ui.tree_node(hash!(hash!(), n), &label, |ui| {
                for (m, zone) in zone.children.iter().enumerate() {
                    zone_ui(ui, zone, n * 1000 + m + 1);
                }
            });
        } else {
            ui.label(None, &label);
        }
    }

    let mut canvas = ui.canvas();
    let w = 515.0;
    let h = 40.0;
    let pos = canvas.request_space(vec2(w, h));

    let rect = Rect::new(pos.x, pos.y, w, h);
    canvas.rect(rect, Color::new(0.5, 0.5, 0.5, 1.0), None);

    let (mouse_x, mouse_y) = mouse_position();

    let mut selected_frame = None;

    // select the slowest frame among the ones close to the mouse cursor
    if rect.contains(vec2(mouse_x, mouse_y)) && state.frames_buffer.len() >= 1 {
        let x = ((mouse_x - pos.x - 2.) / w * FRAMES_BUFFER_CAPACITY as f32) as i32;

        let min = clamp(x - 2, 0, state.frames_buffer.len() as i32 - 1) as usize;
        let max = clamp(x + 3, 0, state.frames_buffer.len() as i32 - 1) as usize;

        selected_frame = state.frames_buffer[min..max]
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.full_frame_time.partial_cmp(&b.full_frame_time).unwrap())
            .map(|(n, _)| n + min);
    }

    if let Some(frame) = selected_frame {
        if is_mouse_button_down(MouseButton::Left) {
            state.selected_frame = state.frames_buffer[frame].try_clone();
        }
    }
    for (n, frame) in state.frames_buffer.iter().enumerate() {
        let x = n as f32 / FRAMES_BUFFER_CAPACITY as f32 * (w - 2.);
        let selected = selected_frame.map_or(false, |selected| n == selected);
        let color = if selected {
            Color::new(1.0, 1.0, 0.0, 1.0)
        } else if frame.full_frame_time < 1.0 / 58.0 {
            Color::new(0.6, 0.6, 1.0, 1.0)
        } else if frame.full_frame_time < 1.0 / 25.0 {
            Color::new(0.3, 0.3, 0.8, 1.0)
        } else {
            Color::new(0.2, 0.2, 0.6, 1.0)
        };
        let t = macroquad::math::clamp(frame.full_frame_time * 1000.0, 0.0, h);

        canvas.line(
            vec2(pos.x + x + 2., pos.y + h - 1.0),
            vec2(pos.x + x + 2., pos.y + h - t),
            color,
        );
    }

    if let Some(frame) = state
        .selected_frame
        .as_ref()
        .or_else(|| state.frames_buffer.get(0))
    {
        ui.label(
            None,
            &format!(
                "Full frame time: {:.3}ms {:.1}(1/t)",
                frame.full_frame_time,
                (1.0 / frame.full_frame_time)
            ),
        );
    }

    if state.paused {
        if ui.button(None, "resume") {
            state.paused = false;
        }
    } else {
        if ui.button(None, "pause") {
            state.paused = true;
        }
    }
    if state.selected_frame.is_some() {
        ui.same_line(100.0);
        if ui.button(None, "deselect frame") {
            state.selected_frame = None;
        }
    }

    let frame = state
        .selected_frame
        .as_ref()
        .or_else(|| state.frames_buffer.get(0));

    ui.separator();
    ui.group(hash!(), vec2(355., 300.), |ui| {
        if let Some(frame) = frame {
            for (n, zone) in frame.zones.iter().enumerate() {
                zone_ui(ui, zone, n + 1);
            }
        }
    });
    ui.group(hash!(), vec2(153., 300.), |ui| {
        let queries = telemetry::gpu_queries();

        for query in queries {
            let t = query.1 as f64 / 1_000_000_000.0;
            ui.label(
                None,
                &format!("{}: {:.3}ms {:.1}(1/t)", query.0, t, 1.0 / t),
            );
        }
    });
    if ui.button(None, "sample gpu") {
        telemetry::sample_gpu_queries();
    }
}

pub fn profiler(params: ProfilerParams) {
    telemetry::pause_gl_capture();
    if storage::try_get::<ProfilerState>().is_none() {
        storage::store(ProfilerState {
            fps_buffer: vec![],
            frames_buffer: vec![],
            profiler_window_opened: false,
            selected_frame: None,
            paused: false,
        })
    }
    let mut state = storage::get_mut::<ProfilerState>();

    let frame = telemetry::frame();

    if state.paused == false && state.profiler_window_opened {
        state.frames_buffer.insert(0, frame);
    }
    let time = get_frame_time();
    state.fps_buffer.insert(0, time);

    state.fps_buffer.truncate(FPS_BUFFER_CAPACITY);
    state.frames_buffer.truncate(FRAMES_BUFFER_CAPACITY);

    push_camera_state();
    set_default_camera();
    let mut sum = 0.0;
    for (x, time) in state.fps_buffer.iter().enumerate() {
        draw_line(
            x as f32 + params.fps_counter_pos.x,
            params.fps_counter_pos.y + 100.0,
            x as f32 + params.fps_counter_pos.x,
            params.fps_counter_pos.y + 100.0 - (time * 2000.0).min(100.0),
            1.0,
            BLUE,
        );
        sum += time;
    }

    let selectable_rect = Rect::new(
        params.fps_counter_pos.x,
        params.fps_counter_pos.y + 40.0,
        100.0,
        100.0,
    );

    if selectable_rect.contains(mouse_position().into()) {
        draw_rectangle(
            selectable_rect.x,
            selectable_rect.y,
            100.0,
            100.0,
            Color::new(1.0, 1.0, 1.0, 0.4),
        );
        if is_mouse_button_pressed(MouseButton::Left) {
            state.profiler_window_opened ^= true;
            if state.profiler_window_opened {
                telemetry::enable();
            } else {
                telemetry::disable();
            }
        }
    }

    draw_text(
        &format!("{:.1}", 1.0 / (sum / state.fps_buffer.len() as f32)),
        params.fps_counter_pos.x,
        params.fps_counter_pos.y + 100.0,
        30.0,
        WHITE,
    );

    if state.profiler_window_opened {
        Window::new(
            hash!(),
            vec2(params.fps_counter_pos.x, params.fps_counter_pos.y + 150.0),
            vec2(525., 450.),
        )
        .ui(&mut *root_ui(), |ui| {
            let tab = ui.tabbar(
                hash!(),
                vec2(300.0, 20.0),
                &["profiler", "scene", "frame", "log"],
            );

            match tab {
                0 => profiler_window(ui, &mut state),
                1 => ui.label(
                    None,
                    &format!(
                        "scene allocated memory: {:.1} kb",
                        (telemetry::scene_allocated_memory() as f32) / 1000.0
                    ),
                ),
                2 => {
                    let drawcalls = telemetry::drawcalls();
                    ui.label(None, &format!("Draw calls: {}", drawcalls.len()));
                    for telemetry::DrawCallTelemetry { indices_count, .. } in &drawcalls {
                        ui.same_line(0.0);

                        ui.label(None, &format!("{}", indices_count));
                        ui.same_line(0.0);
                    }
                    ui.label(None, " ");

                    for telemetry::DrawCallTelemetry {
                        indices_count,
                        texture,
                    } in &drawcalls
                    {
                        ui.label(None, &format!("{}", *indices_count));
                        ui.same_line(0.0);
                        ui.texture(Texture2D::from_miniquad_texture(*texture), 100., 100.0);
                        ui.same_line(0.0);
                    }
                    ui.label(None, " ");

                    if ui.button(None, "Capture frame") {
                        telemetry::capture_frame();
                    }
                }
                3 => {
                    for label in telemetry::strings() {
                        ui.label(None, &label);
                    }
                }
                _ => unreachable!(),
            }
        });
    }
    pop_camera_state();

    telemetry::resume_gl_capture();
}
