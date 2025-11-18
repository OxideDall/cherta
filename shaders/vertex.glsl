attribute vec2 pos;
attribute float t0;
uniform mat4 proj;
uniform float u_now;
uniform float u_ttl;
uniform float u_fade_start;
varying float alpha;
varying float v_t;

void main() {
    gl_Position = proj * vec4(pos, 0.0, 1.0);
    v_t = t0;
    float elapsed = u_now - t0;
    float fade_time = u_ttl - u_fade_start;
    if (elapsed > u_fade_start) {
        alpha = clamp(1.0 - (elapsed - u_fade_start) / fade_time, 0.0, 1.0);
    } else {
        alpha = 1.0;
    }
}

