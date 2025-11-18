precision mediump float;
varying float alpha;
varying float v_t;
uniform vec4 u_color;
uniform float u_feather;

void main() {
    vec4 col = u_color;
    col.rgb *= alpha;
    col.a *= alpha;
    
    // TODO: Real feathering requires quad-based rendering
    // Currently u_feather is not used, but kept for future
    
    gl_FragColor = col;
}

