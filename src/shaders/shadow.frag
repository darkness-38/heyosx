precision mediump float;
varying vec2 v_texcoord;
uniform vec2 size;
uniform float radius;
uniform float blur;
uniform vec4 color;

float rounded_box_sdf(vec2 p, vec2 b, float r) {
    vec2 q = abs(p) - b + r;
    return min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - r;
}

void main() {
    vec2 p = v_texcoord * size;
    vec2 center = size * 0.5;
    float dist = rounded_box_sdf(p - center, (size - blur * 2.0) * 0.5, radius);
    float alpha = 1.0 - smoothstep(-blur, blur, dist);
    gl_FragColor = color * alpha;
}
