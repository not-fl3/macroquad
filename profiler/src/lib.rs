use macroquad::collections::storage;
use macroquad::prelude::*;
use macroquad::telemetry::{self, *};

use megaui_macroquad::{
    draw_megaui, draw_window,
    megaui::{self, hash, Ui},
    WindowParams,
};

pub struct ProfilerState {
    fps_buffer: Vec<f32>,
    profiler_window_opened: bool,
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

pub fn profiler(params: ProfilerParams) {
    if storage::get::<ProfilerState>().is_none() {
        storage::store(ProfilerState {
            fps_buffer: vec![],
            profiler_window_opened: false,
        })
    }

    let frame = profiler_next_frame();
    let queries = telemetry::gpu_queries();

    let mut state = storage::get_mut::<ProfilerState>().unwrap();
    let time = get_frame_time();
    state.fps_buffer.insert(0, time);

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
    if state.fps_buffer.len() > 100 {
        state.fps_buffer.resize(100, 0.0);
    }

    draw_text(
        &format!("{:.1}", 1.0 / (sum / state.fps_buffer.len() as f32)),
        params.fps_counter_pos.x,
        params.fps_counter_pos.y + 100.0,
        30.0,
        WHITE,
    );

    if state.profiler_window_opened {
        fn wtf(ui: &mut Ui, zone: &Zone, n: usize) {
            let label = format!(
                "{}: {:.4}ms {:.1}(1/t)",
                zone.name,
                zone.duration,
                1.0 / zone.duration
            );
            if zone.children.len() != 0 {
                ui.tree_node(hash!(hash!(), n), &label, |ui| {
                    for (m, zone) in zone.children.iter().enumerate() {
                        wtf(ui, zone, n * 1000 + m + 1);
                    }
                });
            } else {
                ui.label(None, &label);
            }
        }

        draw_window(
            hash!(),
            vec2(params.fps_counter_pos.x, params.fps_counter_pos.y + 150.0),
            vec2(520., 420.),
            WindowParams {
                label: "Profiler".to_string(),
                close_button: false,
                titlebar: false,
                ..Default::default()
            },
            |ui| {
                ui.label(
                    None,
                    &format!(
                        "Full frame time: {:.3}ms {:.1}(1/t)",
                        get_frame_time(),
                        (1.0 / get_frame_time())
                    ),
                );

                {
                    let mut canvas = ui.canvas();
                    let w = 500.0;
                    let h = 40.0;
                    let pos = canvas.request_space(megaui::Vector2::new(w, h));

                    canvas.rect(
                        megaui::Rect::new(pos.x, pos.y, w, h),
                        megaui::Color::new(0.5, 0.5, 0.5, 1.0),
                        None,
                    );

                    // TODO: draw nice clickable graph with previous frame time
                }

                ui.separator();
                ui.group(hash!(), megaui::Vector2::new(255., 320.), |ui| {
                    for (n, zone) in frame.zones.iter().enumerate() {
                        wtf(ui, zone, n + 1);
                    }
                });
                ui.group(hash!(), megaui::Vector2::new(255., 320.), |ui| {
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
            },
        );
    }
    draw_megaui();
}
