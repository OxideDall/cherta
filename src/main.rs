mod config;
mod renderer;
mod wayland;

use config::Config;
use renderer::*;
use std::os::raw::{c_int, c_void};
use std::time::Duration;
use wayland::*;
use wayland_client::Connection;

#[link(name = "EGL")]
extern "C" {
    fn eglGetDisplay(native_display: *mut c_void) -> *mut c_void;
    fn eglInitialize(display: *mut c_void, major: *mut c_int, minor: *mut c_int) -> c_int;
    fn eglBindAPI(api: u32) -> c_int;
    fn eglChooseConfig(
        display: *mut c_void,
        attribs: *const c_int,
        configs: *mut *mut c_void,
        config_size: c_int,
        num_config: *mut c_int,
    ) -> c_int;
    fn eglCreateContext(
        display: *mut c_void,
        config: *mut c_void,
        share_context: *mut c_void,
        attribs: *const c_int,
    ) -> *mut c_void;
    fn eglCreateWindowSurface(
        display: *mut c_void,
        config: *mut c_void,
        window: *mut c_void,
        attribs: *const c_int,
    ) -> *mut c_void;
    fn eglMakeCurrent(
        display: *mut c_void,
        draw: *mut c_void,
        read: *mut c_void,
        context: *mut c_void,
    ) -> c_int;
    fn eglSwapBuffers(display: *mut c_void, surface: *mut c_void) -> c_int;
}

const EGL_OPENGL_ES_API: u32 = 0x30A0;

fn main() {
    let config = Config::load();

    let conn = Connection::connect_to_env().unwrap();
    let mut event_queue = conn.new_event_queue();
    let mut state = WaylandState::new();

    setup_wayland(&mut state, &conn, &mut event_queue);

    let surface = state.surface.as_ref().unwrap();
    let wl_egl_window = create_egl_window(surface, state.width, state.height);

    let egl_display = unsafe {
        let native_display = conn.backend().display_ptr() as *mut c_void;
        eglGetDisplay(native_display)
    };

    unsafe {
        eglInitialize(egl_display, std::ptr::null_mut(), std::ptr::null_mut());
        eglBindAPI(EGL_OPENGL_ES_API);

        let config_attribs: [c_int; 11] = [
            0x3024, 8, 0x3023, 8, 0x3022, 8, 0x3021, 8, 0x3025, 0, 0x3038,
        ];

        let mut egl_config: *mut c_void = std::ptr::null_mut();
        let mut num_config = 0;
        eglChooseConfig(
            egl_display,
            config_attribs.as_ptr(),
            &mut egl_config,
            1,
            &mut num_config,
        );

        let ctx_attribs: [c_int; 3] = [0x3098, 2, 0x3038];
        let egl_context = eglCreateContext(
            egl_display,
            egl_config,
            std::ptr::null_mut(),
            ctx_attribs.as_ptr(),
        );

        let egl_surface = eglCreateWindowSurface(
            egl_display,
            egl_config,
            wl_egl_window.ptr() as *mut c_void,
            std::ptr::null(),
        );

        eglMakeCurrent(egl_display, egl_surface, egl_surface, egl_context);

        glViewport(0, 0, state.width, state.height);
        glEnable(GL_BLEND);
        glBlendFunc(GL_ONE, GL_ONE_MINUS_SRC_ALPHA);

        glEnable(GL_LINE_SMOOTH);
        glHint(GL_LINE_SMOOTH_HINT, GL_NICEST);

        let vertex_src = include_str!("../shaders/vertex.glsl");
        let fragment_src = include_str!("../shaders/fragment.glsl");
        let program = create_program(vertex_src, fragment_src).expect("Shader compilation failed");
        glUseProgram(program);

        let proj_loc = glGetUniformLocation(program, b"proj\0".as_ptr());
        let u_now_loc = glGetUniformLocation(program, b"u_now\0".as_ptr());
        let u_ttl_loc = glGetUniformLocation(program, b"u_ttl\0".as_ptr());
        let u_fade_start_loc = glGetUniformLocation(program, b"u_fade_start\0".as_ptr());
        let u_color_loc = glGetUniformLocation(program, b"u_color\0".as_ptr());
        let u_feather_loc = glGetUniformLocation(program, b"u_feather\0".as_ptr());

        let proj = ortho_matrix(state.width as f32, state.height as f32);
        glUniformMatrix4fv(proj_loc, 1, 0, proj.as_ptr());

        let pos_loc = glGetAttribLocation(program, b"pos\0".as_ptr());
        let t0_loc = glGetAttribLocation(program, b"t0\0".as_ptr());

        loop {
            if matches!(state.input_state, wayland::InputState::Passthrough) {
                let can_poll = if let Some(last_scroll) = state.last_scroll {
                    last_scroll.elapsed().as_millis() > config.scroll_cooldown as u128
                } else {
                    true
                };

                if can_poll
                    && state.last_poll.elapsed().as_millis() > config.polling_interval as u128
                {
                    state.input_state = wayland::InputState::Capturing;
                    state.set_input_passthrough(false);
                    state.last_poll = std::time::Instant::now();
                }
            }

            event_queue.dispatch_pending(&mut state).unwrap();

            glClearColor(0.0, 0.0, 0.0, 0.0);
            glClear(GL_COLOR_BUFFER_BIT);

            glLineWidth(config.thickness);
            glUniform1f(u_ttl_loc, config.ttl);
            glUniform1f(u_fade_start_loc, config.fade_start);
            glUniform1f(u_feather_loc, config.line_feather);
            glUniform4f(
                u_color_loc,
                config.color[0],
                config.color[1],
                config.color[2],
                config.opacity,
            );

            let now = state.start_time.elapsed().as_secs_f32();
            glUniform1f(u_now_loc, now);

            state.strokes.retain(|stroke| {
                if stroke.is_empty() {
                    return false;
                }
                let age = now - stroke.last().unwrap().2;
                age < config.ttl
            });

            for stroke in &state.strokes {
                if stroke.len() < 2 {
                    continue;
                }

                let points = if config.smooth_lines {
                    smooth_points(stroke, config.min_point_distance)
                } else {
                    stroke.clone()
                };

                if points.len() < 2 {
                    continue;
                }

                let mut vbo = 0u32;
                glGenBuffers(1, &mut vbo);
                glBindBuffer(GL_ARRAY_BUFFER, vbo);
                glBufferData(
                    GL_ARRAY_BUFFER,
                    (points.len() * 12) as isize,
                    points.as_ptr() as *const c_void,
                    GL_STATIC_DRAW,
                );

                glEnableVertexAttribArray(pos_loc as u32);
                glVertexAttribPointer(pos_loc as u32, 2, GL_FLOAT, 0, 12, std::ptr::null());

                glEnableVertexAttribArray(t0_loc as u32);
                glVertexAttribPointer(t0_loc as u32, 1, GL_FLOAT, 0, 12, 8 as *const c_void);

                glDrawArrays(GL_LINE_STRIP, 0, points.len() as c_int);
            }

            if !state.current_stroke.is_empty() {
                let points = if config.smooth_lines {
                    smooth_points(&state.current_stroke, config.min_point_distance)
                } else {
                    state.current_stroke.clone()
                };

                if points.len() >= 2 {
                    let mut vbo = 0u32;
                    glGenBuffers(1, &mut vbo);
                    glBindBuffer(GL_ARRAY_BUFFER, vbo);
                    glBufferData(
                        GL_ARRAY_BUFFER,
                        (points.len() * 12) as isize,
                        points.as_ptr() as *const c_void,
                        GL_STATIC_DRAW,
                    );

                    glEnableVertexAttribArray(pos_loc as u32);
                    glVertexAttribPointer(pos_loc as u32, 2, GL_FLOAT, 0, 12, std::ptr::null());

                    glEnableVertexAttribArray(t0_loc as u32);
                    glVertexAttribPointer(t0_loc as u32, 1, GL_FLOAT, 0, 12, 8 as *const c_void);

                    glDrawArrays(GL_LINE_STRIP, 0, points.len() as c_int);
                }
            }

            eglSwapBuffers(egl_display, egl_surface);

            std::thread::sleep(Duration::from_millis(16));
        }
    }
}
