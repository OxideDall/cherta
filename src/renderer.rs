use std::os::raw::{c_int, c_void};

#[link(name = "GLESv2")]
extern "C" {
    pub fn glClearColor(r: f32, g: f32, b: f32, a: f32);
    pub fn glClear(mask: u32);
    pub fn glViewport(x: c_int, y: c_int, w: c_int, h: c_int);
    pub fn glEnable(cap: u32);
    pub fn glBlendFunc(sfactor: u32, dfactor: u32);
    pub fn glLineWidth(width: f32);
    pub fn glHint(target: u32, mode: u32);
    pub fn glCreateShader(shader_type: u32) -> u32;
    pub fn glShaderSource(
        shader: u32,
        count: c_int,
        string: *const *const u8,
        length: *const c_int,
    );
    pub fn glCompileShader(shader: u32);
    pub fn glGetShaderiv(shader: u32, pname: u32, params: *mut c_int);
    pub fn glGetShaderInfoLog(shader: u32, bufsize: c_int, length: *mut c_int, infolog: *mut u8);
    pub fn glCreateProgram() -> u32;
    pub fn glAttachShader(program: u32, shader: u32);
    pub fn glLinkProgram(program: u32);
    pub fn glGetProgramiv(program: u32, pname: u32, params: *mut c_int);
    pub fn glUseProgram(program: u32);
    pub fn glGetUniformLocation(program: u32, name: *const u8) -> c_int;
    pub fn glUniform1f(location: c_int, v0: f32);
    pub fn glUniform4f(location: c_int, v0: f32, v1: f32, v2: f32, v3: f32);
    pub fn glUniformMatrix4fv(location: c_int, count: c_int, transpose: u8, value: *const f32);
    pub fn glGenBuffers(n: c_int, buffers: *mut u32);
    pub fn glBindBuffer(target: u32, buffer: u32);
    pub fn glBufferData(target: u32, size: isize, data: *const c_void, usage: u32);
    pub fn glGetAttribLocation(program: u32, name: *const u8) -> c_int;
    pub fn glEnableVertexAttribArray(index: u32);
    pub fn glVertexAttribPointer(
        index: u32,
        size: c_int,
        type_: u32,
        normalized: u8,
        stride: c_int,
        pointer: *const c_void,
    );
    pub fn glDrawArrays(mode: u32, first: c_int, count: c_int);
}

pub const GL_COLOR_BUFFER_BIT: u32 = 0x4000;
pub const GL_BLEND: u32 = 0x0BE2;
pub const GL_ONE: u32 = 1;
pub const GL_ONE_MINUS_SRC_ALPHA: u32 = 0x0303;
pub const GL_LINE_SMOOTH: u32 = 0x0B20;
pub const GL_LINE_SMOOTH_HINT: u32 = 0x0C52;
pub const GL_NICEST: u32 = 0x1102;
pub const GL_VERTEX_SHADER: u32 = 0x8B31;
pub const GL_FRAGMENT_SHADER: u32 = 0x8B30;
pub const GL_ARRAY_BUFFER: u32 = 0x8892;
pub const GL_STATIC_DRAW: u32 = 0x88E4;
pub const GL_FLOAT: u32 = 0x1406;
pub const GL_LINE_STRIP: u32 = 0x0003;
pub const GL_COMPILE_STATUS: u32 = 0x8B81;
pub const GL_LINK_STATUS: u32 = 0x8B82;

pub fn compile_shader(src: &str, shader_type: u32) -> Result<u32, String> {
    unsafe {
        let shader = glCreateShader(shader_type);
        let c_str = std::ffi::CString::new(src).unwrap();
        let ptr = c_str.as_ptr() as *const u8;
        glShaderSource(shader, 1, &ptr, std::ptr::null());
        glCompileShader(shader);

        let mut status = 0;
        glGetShaderiv(shader, GL_COMPILE_STATUS, &mut status);
        if status == 0 {
            let mut log_len = 0;
            glGetShaderiv(shader, 0x8B84, &mut log_len);
            let mut log = vec![0u8; log_len as usize];
            glGetShaderInfoLog(shader, log_len, std::ptr::null_mut(), log.as_mut_ptr());
            return Err(String::from_utf8_lossy(&log).to_string());
        }
        Ok(shader)
    }
}

pub fn create_program(vs_src: &str, fs_src: &str) -> Result<u32, String> {
    unsafe {
        let vs = compile_shader(vs_src, GL_VERTEX_SHADER)?;
        let fs = compile_shader(fs_src, GL_FRAGMENT_SHADER)?;
        let prog = glCreateProgram();
        glAttachShader(prog, vs);
        glAttachShader(prog, fs);
        glLinkProgram(prog);

        let mut status = 0;
        glGetProgramiv(prog, GL_LINK_STATUS, &mut status);
        if status == 0 {
            return Err("Failed to link program".to_string());
        }
        Ok(prog)
    }
}

pub fn ortho_matrix(width: f32, height: f32) -> [f32; 16] {
    let mut m = [0.0f32; 16];
    m[0] = 2.0 / width;
    m[5] = -2.0 / height;
    m[10] = -1.0;
    m[12] = -1.0;
    m[13] = 1.0;
    m[15] = 1.0;
    m
}

// Простое сглаживание через децимацию близких точек
pub fn smooth_points(points: &[(f32, f32, f32)], min_distance: f32) -> Vec<(f32, f32, f32)> {
    if points.len() < 2 {
        return points.to_vec();
    }

    let mut result = vec![points[0]];
    for point in points.iter().skip(1) {
        let last = result.last().unwrap();
        let dist = ((point.0 - last.0).powi(2) + (point.1 - last.1).powi(2)).sqrt();
        if dist >= min_distance {
            result.push(*point);
        }
    }

    // Всегда добавляем последнюю точку
    if result.last() != Some(&points[points.len() - 1]) {
        result.push(points[points.len() - 1]);
    }

    result
}
