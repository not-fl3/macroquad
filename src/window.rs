//! Window and associated to window rendering context related functions.

use crate::get_context;

use crate::color::Color;

// miniquad is re-exported for the use in combination with `get_internal_gl`
pub use miniquad;

pub use miniquad::conf::Conf;

/// Block execution until the next frame.
pub fn next_frame() -> crate::exec::FrameFuture {
    crate::exec::FrameFuture::default()
}

/// Fill window background with solid color.
/// Note: even when "clear_background" was not called explicitly
/// screen will be cleared at the beginning of the frame.
pub fn clear_background(color: Color) {
    let context = get_context();

    context.gl.clear(&mut context.quad_context, color);
}

#[doc(hidden)]
pub fn gl_set_drawcall_buffer_capacity(max_vertices: usize, max_indices: usize) {
    let context = get_context();
    context
        .gl
        .update_drawcall_capacity(&mut context.quad_context, max_vertices, max_indices);
}

pub struct InternalGlContext<'a> {
    pub quad_context: &'a mut miniquad::Context,
    pub quad_gl: &'a mut crate::quad_gl::QuadGl,
}

impl<'a> InternalGlContext<'a> {
    /// Draw all the batched stuff and reset the internal state cache
    /// May be helpful for combining macroquad's drawing with raw miniquad/opengl calls
    pub fn flush(&mut self) {
        get_context().perform_render_passes();
    }
}

pub unsafe fn get_internal_gl<'a>() -> InternalGlContext<'a> {
    let context = get_context();

    InternalGlContext {
        quad_context: &mut context.quad_context,
        quad_gl: &mut context.gl,
    }
}

pub fn screen_width() -> f32 {
    let context = get_context();

    context.screen_width / context.quad_context.dpi_scale()
}

pub fn screen_height() -> f32 {
    let context = get_context();

    context.screen_height / context.quad_context.dpi_scale()
}

/// Request the window size to be the given value. This takes DPI into account.
///
/// Note that the OS might decide to give a different size. Additionally, the size in macroquad won't be updated until the next `next_frame().await`.
pub fn request_new_screen_size(width: f32, height: f32) {
    let context = get_context();
    context.quad_context.set_window_size(
        (width * context.quad_context.dpi_scale()) as u32,
        (height * context.quad_context.dpi_scale()) as u32,
    );
    // We do not set the context.screen_width and context.screen_height here.
    // After `set_window_size` is called, EventHandlerFree::resize will be invoked, setting the size correctly.
    // Because the OS might decide to give a different screen dimension, setting the context.screen_* here would be confusing.
}

/// With `set_panic_handler` set to a handler code, macroquad will use
/// `std::panic::catch_unwind` on user code to catch some panics.
///
/// Sometimes it is nice to let player send a bug report with a screenshot of an
/// error. It is way easier to ask for a screenshot than ask to connect to a phone
/// with adb and post a log.
///
/// For this very case "set_panic_handler" exists.
/// ```ignore
/// set_panic_handler(|msg, backtrace| async move {
///     loop {
///         clear_background(RED);
///         ui::root_ui().label(None, &msg);
///         for line in backtrace.split('\n') {
///             root_ui().label(None, line);
///         }
///         next_frame().await;
///      }
/// });
/// ```
///
/// `set_panic_handler` acts as a second app entry-point, that will be used
/// after a panic in user code will happen. Macroquad will also try to catch some OS
/// panics, but not all of them - some compatibility bugs may end up crashing the app.
///
/// Withot `set_panic_handler` macroquad will not use `catch_unwind` at all,
/// therefore `panic_handler` is completely optional.
/// NOTE: only with "backtrace" macroquad feature `backtrace` string will contain an
/// actual backtrace. Otherwise only panic location and message will be available.
/// NOTE: on android, even with "backtrace" nice backtrace is available only if the game is compiled with sdk >= 21.
/// To use sdk >= 21 add "min_sdk_version = 21" to Cargo.toml
pub fn set_panic_handler<T, F>(future: F)
where
    T: std::future::Future<Output = ()> + 'static,
    F: Fn(String, String) -> T + Send + Sync + 'static,
{
    std::panic::set_hook(Box::new(move |info| {
        let message = format!("{:?}", info);
        #[cfg(feature = "backtrace")]
        let backtrace_string = format!("{:?}", backtrace::Backtrace::new());
        #[cfg(not(feature = "backtrace"))]
        let backtrace_string = format!("Macroquad compiled without \"backtrace\" feature");
        crate::logging::error!("{}", message);
        crate::logging::error!("{}", backtrace_string);

        crate::get_context().recovery_future = Some(Box::pin(future(message, backtrace_string)));
    }));

    crate::get_context().unwind = true;
}
